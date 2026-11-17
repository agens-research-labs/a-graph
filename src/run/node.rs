//! Running one node: the prompt with its placeholders filled, the model called, the answer
//! delivered where the node says, with the loop of tools, MCP servers and skills in between. Not yet:
//! what a node reads from vectordb, search and memory.

use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;

use super::Program;
use super::utility::Item;
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::CHUNKS;
use crate::graph::Entity;
use crate::graph::nodes::component::{InFrom, MediaIn, OutFrom, Validation};
use crate::models::{
    MediaInput, MediaOutput, ModelInvocation, ModelResponse, ProducedMedia, ResponseInfo, StopReason, TokenLogprob,
};
use crate::resources::{MediaKind, Origin, PromptBody, Source};

/// What a node receives.
#[derive(Debug, Clone, Default)]
pub struct NodeInput {
    /// The message of the user, when the node takes it (`user_message`).
    pub user_message: Option<String>,
    /// The message of the connected node, when the node takes it (`agent_message`).
    pub agent_message: Option<String>,
    /// The media of the connected node, when the node takes them (`media.input.node`).
    pub agent_media: Vec<MediaOutput>,
}

/// A message: written text, or JSON when the node has an output schema.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Text(String),
    Json(Value),
}

impl Content {
    pub(super) fn text(&self) -> String {
        match self {
            Content::Text(t) => t.clone(),
            Content::Json(j) => j.to_string(),
        }
    }
}

/// What goes to one destination.
#[derive(Debug, Clone, Default)]
pub struct Delivery {
    pub message: Option<Content>,
    pub media: Vec<MediaOutput>,
    /// The reasoning of the model; only the final output has it, it never goes to a node.
    pub reasoning: Vec<Value>,
    /// The tokens of the answer with their log probabilities, one list for each call to the model; only the
    /// final output has them, they never go to a node.
    pub logprobs: Vec<Vec<TokenLogprob>>,
}

#[derive(Debug, Clone)]
pub struct NodeOutput {
    /// To the final output list (`assistant_message`, `assistant_media`, `reasoning`).
    pub assistant: Delivery,
    /// To the connected node (`node`).
    pub node: Delivery,
    pub stop_reason: StopReason,
    pub info: ResponseInfo,
}

/// A node ready to call the model: the request, built from the node and its input.
pub(super) struct Prepared {
    pub(super) model: String,
    pub(super) call: ModelInvocation,
    /// The text the node uses, after `message.input.utility`.
    pub(super) input: NodeInput,
}

impl Program {
    /// Runs one node. The errors are of the execution and carry the id of the node.
    pub async fn run_node(&self, node: Entity, input: NodeInput) -> Result<NodeOutput, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        if self.nodes.validation().get(node).is_some() {
            return Err(fail(
                Rule::NotSupported,
                "a validation node is run by `run`: its answer is a verdict, not an output".into(),
            ));
        }
        let prepared = self.prepare_node(node, &input).await?;
        let llm = self
            .models
            .get(&prepared.model)
            .ok_or_else(|| fail(Rule::UnknownRef, format!("model `{}` is not built", prepared.model)))?;
        let response = self.invoke_with_tools(node, llm, prepared.call).await?;
        self.finish_node(node, &prepared.input, response).await
    }

    /// The first half of running a node: everything up to the call to the model.
    pub(super) async fn prepare_node(&self, node: Entity, input: &NodeInput) -> Result<Prepared, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let transformed = self.transform_input(node, input).await?;
        let input = &transformed;

        let Some(model) = self.nodes.model().get(node) else {
            return Err(fail(Rule::MissingField, "the node has no `model`".into()));
        };
        let uses_search = self.nodes.external().get(node).is_some_and(|e| !e.search.is_empty());
        if uses_search {
            return Err(fail(Rule::NotSupported, "search cannot be run yet".into()));
        }
        // the servers are connected by `run`: a node that uses them cannot run alone
        let needs_mcp = self
            .nodes
            .resources()
            .get(node)
            .is_some_and(|r| !r.mcp_servers.is_empty());
        if needs_mcp && self.mcp_pool().is_none() {
            return Err(fail(
                Rule::NotSupported,
                "the MCP servers are connected by `run`: a node that uses them cannot be run alone".into(),
            ));
        }

        // the values the placeholders take
        let wants = self.nodes.message().get(node).and_then(|m| m.input).unwrap_or_default();
        let mut values: HashMap<String, String> = HashMap::new();
        let mut texts: Vec<String> = Vec::new();
        for (enabled, name, given) in [
            (wants.user_message, "user_message", &input.user_message),
            (wants.agent_message, "agent_message", &input.agent_message),
        ] {
            if enabled {
                let text = given.clone().ok_or_else(|| {
                    fail(
                        Rule::MissingField,
                        format!("the node takes `{name}` and none was given"),
                    )
                })?;
                values.insert(name.into(), text.clone());
                texts.push(text);
            }
        }
        if wants.user_message != wants.agent_message {
            values.insert("input_message".into(), texts[0].clone());
        }
        if self.nodes.external().get(node).is_some_and(|e| !e.vectordb.is_empty()) {
            values.insert(CHUNKS.into(), self.read_vectordbs(node, input).await?);
        }
        let memory = self.read_memories(node, input).await?;
        values.extend(memory.synt);
        let board = self.nodes.blackboard().get(node);
        if let Some(b) = board.filter(|b| b.read) {
            values.insert(b.id.clone(), self.blackboards.read(&b.id)?);
        }

        // the prompt; without one, the user text is the messages the node takes
        let (system, user) = match self.nodes.resources().get(node).and_then(|r| r.prompt.as_deref()) {
            Some(pid) => match self.resources.prompts.get(pid).map(|p| &p.body) {
                Some(PromptBody::Content(c)) => (
                    fill(&c.system, &values).map_err(|e| fail(Rule::PlaceholderUnbound, e))?,
                    fill(&c.user, &values).map_err(|e| fail(Rule::PlaceholderUnbound, e))?,
                ),
                _ => {
                    return Err(fail(
                        Rule::NotSupported,
                        format!("prompt `{pid}` has no text yet (a prompt from a url is not read)"),
                    ));
                }
            },
            None => (String::new(), texts.join("\n\n")),
        };

        // the skills of the node: how to use them, and their names and descriptions
        let system = match self.skills_catalog(node) {
            Some(catalog) if system.is_empty() => catalog,
            Some(catalog) => format!("{system}\n\n{catalog}"),
            None => system,
        };

        // the media the node reads, and the ones it asks to produce
        let mut media = Vec::new();
        let mut output_media = Vec::new();
        if let Some(m) = self.nodes.media().get(node) {
            for entry in &m.input {
                media.push(self.read_media(&id, entry, input).await?);
            }
            // the model makes the ones that are `from: model`; the others a utility makes from the text
            for entry in m.output.iter().filter(|o| o.from == OutFrom::Model) {
                if let Some(Origin::Generated { generate, provider }) =
                    self.resources.media.get(&entry.id).map(|m| &m.origin)
                {
                    let kind = self
                        .resources
                        .media
                        .get(&entry.id)
                        .map(|m| m.kind.clone())
                        .unwrap_or(MediaKind::Md);
                    output_media.push(ProducedMedia {
                        kind,
                        generate: generate.clone(),
                        provider: provider.clone(),
                    });
                }
            }
        }

        let call = ModelInvocation {
            system,
            history: memory.history,
            user,
            media,
            tools: self.tool_definitions(node),
            output_media,
            // the answer of a judge is fixed
            schema: match self.nodes.validation().get(node) {
                Some(_) => Some(Validation::judge_schema()),
                None => self
                    .nodes
                    .message()
                    .get(node)
                    .and_then(|m| m.output.as_ref())
                    .and_then(|o| o.schema.clone()),
            },
            temperature: model.temperature,
            top_p: model.top_p,
            max_tokens: model.max_tokens,
            reasoning: model.reasoning,
            logprobs: self
                .nodes
                .message()
                .get(node)
                .and_then(|m| m.output.as_ref())
                .and_then(|o| o.logprobs),
            tool_rounds: Vec::new(),
        };
        self.node_started(node);
        Ok(Prepared {
            model: model.id.clone(),
            call,
            input: transformed.clone(),
        })
    }

    /// The second half: the answer of the model goes where the node says, and the blackboard is
    /// written and cleaned.
    pub(super) async fn finish_node(
        &self,
        node: Entity,
        input: &NodeInput,
        response: ModelResponse,
    ) -> Result<NodeOutput, Error> {
        self.node_finished(node, &response, None);
        let board = self.nodes.blackboard().get(node);

        // the answer goes where the node says
        let content = response
            .structured
            .clone()
            .map(Content::Json)
            .or_else(|| response.message.clone().map(Content::Text));
        // `message.output.utility`: the text is changed once, then every destination gets the new one
        let content = self.transform_output(node, content).await?;
        let (mut assistant, mut to_node) = (Delivery::default(), Delivery::default());
        if let Some(o) = self.nodes.message().get(node).and_then(|m| m.output.as_ref()) {
            if o.assistant_message {
                assistant.message = content.clone();
            }
            if o.node {
                to_node.message = content.clone();
            }
            if o.reasoning {
                assistant.reasoning = response.reasoning.clone();
            }
            // those of the answer the model gave, before `message.output.utility`
            if o.logprobs.is_some() {
                assistant.logprobs = response.logprobs.clone();
            }
        }
        // the media: the ones the model made (in the order asked) and the ones a utility makes from the text
        let entries = self
            .nodes
            .media()
            .get(node)
            .map(|m| m.output.clone())
            .unwrap_or_default();
        let name = self.nodes.id(node).unwrap_or_default().to_string();
        let mut made_by_model = response.media.clone().into_iter();
        for entry in &entries {
            let made = match entry.from {
                OutFrom::Model => made_by_model.next().map(|mut m| {
                    m.id = Some(entry.id.clone());
                    m
                }),
                OutFrom::Message => {
                    let text = content.as_ref().map(Content::text).ok_or_else(|| {
                        Error::new(
                            Phase::Execution,
                            Path::node(&name),
                            Rule::MissingField,
                            format!(
                                "media `{}` is made from the text of the node, which gave none",
                                entry.id
                            ),
                        )
                    })?;
                    match self
                        .item_through(&name, "media.output", &entry.utility, Item::Text(text))
                        .await?
                    {
                        Item::Data { mime_type, data } => Some(MediaOutput {
                            mime_type,
                            data,
                            id: Some(entry.id.clone()),
                        }),
                        _ => {
                            return Err(Error::new(
                                Phase::Execution,
                                Path::node(&name),
                                Rule::UtilityFailed,
                                format!("the utilities of media `{}` must end with a file", entry.id),
                            ));
                        }
                    }
                }
            };
            let Some(made) = made else { continue };
            let made = if entry.from == OutFrom::Model {
                self.media_through(&name, &entry.utility, made).await?
            } else {
                made
            };
            if entry.assistant {
                assistant.media.push(made.clone());
            }
            if entry.node {
                to_node.media.push(made);
            }
        }

        // the blackboard: what the node wrote, then the cleanup once the task is done
        if let Some(b) = board {
            if let (Some(w), Some(c)) = (b.write.as_ref().filter(|w| w.enabled), &content) {
                // the text written is the one delivered, changed once more if the blackboard asks for it
                let text = self
                    .text_through(
                        self.nodes.id(node).unwrap_or_default(),
                        "blackboard.write",
                        &w.utility,
                        c.text(),
                    )
                    .await?;
                self.blackboards.write(&b.id, &text, w.place).await?;
            }
            if b.cleanup {
                self.blackboards.cleanup(&b.id).await?;
            }
        }
        // the memories: the exchange of this turn is saved, and the summary is made when it is time
        self.write_memories(node, input, content.as_ref()).await?;
        Ok(NodeOutput {
            assistant,
            node: to_node,
            stop_reason: response.stop_reason,
            info: response.info,
        })
    }

    /// One media the node reads: as it is, or changed by its utilities first.
    async fn read_media(&self, node: &str, entry: &MediaIn, input: &NodeInput) -> Result<MediaInput, Error> {
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(node), rule, msg);
        let start = match entry.from {
            InFrom::Resources => {
                if entry.utility.is_empty() {
                    return self.load_media(&entry.id).map_err(|e| fail(Rule::Io, e));
                }
                match self.resources.media.get(&entry.id).map(|m| &m.origin) {
                    Some(Origin::Source(Source::Path(p))) => Item::Path(p.clone()),
                    Some(Origin::Source(Source::Base64 { data, mime_type })) => Item::Base64 {
                        mime_type: mime_type.clone(),
                        data: data.clone(),
                    },
                    // a url is not read yet: the node fails as it does without utilities
                    Some(Origin::Source(Source::Url { .. })) => {
                        return Err(fail(
                            Rule::Io,
                            format!("media `{}` is a url, which is not read yet", entry.id),
                        ));
                    }
                    None => {
                        return Err(fail(Rule::Io, format!("media `{}` is not declared", entry.id)));
                    }
                    Some(_) => {
                        return Err(fail(
                            Rule::Io,
                            format!("media `{}` is produced by a node, it cannot be read", entry.id),
                        ));
                    }
                }
            }
            InFrom::Node => {
                let given = input
                    .agent_media
                    .iter()
                    .find(|m| m.id.as_deref() == Some(entry.id.as_str()));
                let Some(given) = given else {
                    return Err(fail(
                        Rule::MissingField,
                        format!("the node before gave no media `{}`", entry.id),
                    ));
                };
                if entry.utility.is_empty() {
                    return image_of(given).map_err(|e| fail(Rule::NotSupported, e));
                }
                Item::Data {
                    mime_type: given.mime_type.clone(),
                    data: given.data.clone(),
                }
            }
        };
        match self.item_through(node, "media.input", &entry.utility, start).await? {
            Item::Text(t) => Ok(MediaInput::Markdown(t)),
            Item::Data { mime_type, data } => match mime_type.strip_prefix("image/") {
                Some(format) => Ok(MediaInput::Image {
                    format: format.to_string(),
                    data,
                }),
                None => Err(fail(
                    Rule::UtilityFailed,
                    format!(
                        "media `{}`: the utilities gave a file of type `{mime_type}`, which a model cannot read: they must end with a text or an image",
                        entry.id
                    ),
                )),
            },
            Item::Path(_) | Item::Base64 { .. } => Err(fail(
                Rule::UtilityFailed,
                format!("media `{}`: the utilities gave no result", entry.id),
            )),
        }
    }

    /// A media resource the node reads: a file or a base64 text. A url is not read yet.
    fn load_media(&self, id: &str) -> Result<MediaInput, String> {
        let media = self
            .resources
            .media
            .get(id)
            .ok_or_else(|| format!("media `{id}` is not declared"))?;
        let Origin::Source(source) = &media.origin else {
            return Err(format!("media `{id}` is produced by a node, it cannot be read"));
        };
        match (&media.kind, source) {
            (MediaKind::Md, Source::Path(p)) => std::fs::read_to_string(p)
                .map(MediaInput::Markdown)
                .map_err(|e| format!("cannot read `{p}`: {e}")),
            (MediaKind::Img(f), Source::Path(p)) => std::fs::read(p)
                .map(|b| MediaInput::Image {
                    format: f.clone(),
                    data: STANDARD.encode(b),
                })
                .map_err(|e| format!("cannot read `{p}`: {e}")),
            (MediaKind::Img(f), Source::Base64 { data: b, .. }) => Ok(MediaInput::Image {
                format: f.clone(),
                data: b.clone(),
            }),
            (MediaKind::Md, Source::Base64 { data: b, .. }) => {
                let bytes = STANDARD
                    .decode(b)
                    .map_err(|e| format!("media `{id}` is not valid base64: {e}"))?;
                String::from_utf8(bytes)
                    .map(MediaInput::Markdown)
                    .map_err(|e| format!("media `{id}` is not text: {e}"))
            }
            (_, Source::Url { .. }) => Err(format!("media `{id}` is a url, which is not read yet")),
            (MediaKind::Audio(_) | MediaKind::Video(_), _) => {
                Err(format!("media `{id}`: audio and video input are not supported yet"))
            }
            (MediaKind::File, _) => Err(format!(
                "media `{id}` is a file of any format: a utility has to change it into text"
            )),
        }
    }
}

/// A media of the connected node as an input: only images for now.
fn image_of(m: &MediaOutput) -> Result<MediaInput, String> {
    match m.mime_type.strip_prefix("image/") {
        Some(format) => Ok(MediaInput::Image {
            format: format.to_string(),
            data: m.data.clone(),
        }),
        None => Err(format!(
            "a media of type `{}` cannot be an input yet (images only)",
            m.mime_type
        )),
    }
}

/// The call asked the log probabilities and the answer has a text without them: the node fails, so
/// that an output never lacks them unnoticed. A refusal is not the answer the model wrote, and
/// has none; an answer without text has nothing to give them for.
pub(super) fn logprobs_missing(call: &ModelInvocation, response: &ModelResponse) -> Option<String> {
    let has_text = response.message.is_some() || response.structured.is_some();
    let missing = call.logprobs.is_some()
        && has_text
        && response.stop_reason != StopReason::Refused
        && response.logprobs.iter().all(Vec::is_empty);
    missing.then(|| "logprobs were asked and the model gave none with its answer".to_string())
}

/// Replaces every `{{name}}` with its value.
pub(super) fn fill(text: &str, values: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("}}").ok_or("unclosed `{{`")?;
        let name = after[..end].trim();
        out.push_str(
            values
                .get(name)
                .ok_or_else(|| format!("{{{{{name}}}}} has no value on this node"))?,
        );
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}
