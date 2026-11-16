//! What a node reads from its memories: the exchanges go to the model as the history, the summary
//! goes into the prompt where the node writes `{{<memory-id>}}`.

use std::collections::HashMap;

use super::Program;
use super::node::{Content, NodeInput, fill};
use crate::error::{Error, Path, Phase, Rule};
use crate::external::{CHAT_PLACEHOLDER, Entry, Memory, MemoryBackend, Record, SYNT_PLACEHOLDER, Take, read_record};
use crate::graph::Entity;
use crate::logging::{Event, LogLevel};
use crate::models::{ChatEntry, ModelInvocation, Role};
use crate::resources::PromptBody;

/// What the memories of a node give to a call.
pub(super) struct MemoryRead {
    pub(super) history: Vec<ChatEntry>,
    /// The summary of every memory that has a `synt`, by memory id.
    pub(super) synt: HashMap<String, String>,
}

impl Program {
    pub(super) async fn read_memories(&self, node: Entity, input: &NodeInput) -> Result<MemoryRead, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let mut read = MemoryRead {
            history: Vec::new(),
            synt: HashMap::new(),
        };
        let Some(external) = self.nodes.external().get(node) else {
            return Ok(read);
        };

        // the message the node takes is what the exchanges are compared with
        let query = self.taken(node, input);
        for m in &external.memory {
            let began = std::time::Instant::now();
            let memory = self
                .external
                .memory
                .get(&m.id)
                .ok_or_else(|| fail(Rule::UnknownRef, format!("memory `{}` is not built", m.id)))?;
            let MemoryBackend::JsonFile { path } = &memory.backend else {
                return Err(fail(
                    Rule::NotSupported,
                    format!("memory `{}` is kept in a vectordb, which is not supported yet", m.id),
                ));
            };
            let record = read_record(path, &id)
                .map_err(|e| fail(Rule::Io, format!("memory `{}`: {e}", m.id)).with_detail_if(e.detail.as_deref()))?;
            if memory.synt.is_some() {
                let summary = self
                    .text_through(&id, "memory.read", &m.read_utility, record.synt.clone())
                    .await?;
                read.synt.insert(m.id.clone(), summary);
            }
            let Some(chat) = &memory.chat else { continue };
            let vector = match (&chat.embedding, &query) {
                (Some(embedding), Some(text)) => {
                    let mut vectors = self
                        .embeddings
                        .embed(embedding, std::slice::from_ref(text))
                        .await
                        .map_err(|e| fail(Rule::ModelFailed, e.to_string()).with_detail_if(e.said()))?;
                    vectors.pop()
                }
                _ => None,
            };
            let entries = record.select(
                chat,
                vector.as_deref(),
                Take {
                    user: m.read.user,
                    assistant: m.read.assistant,
                },
            );
            self.emit(
                self.traced(
                    Event::new(LogLevel::Debug, "read.memory")
                        .node(&id)
                        .field("id", m.id.clone())
                        .field("messages", entries.len())
                        .field("summary", memory.synt.is_some() && !record.synt.is_empty())
                        .field("duration", began.elapsed().as_secs_f64()),
                ),
            );
            for e in entries {
                let content = self
                    .text_through(&id, "memory.read", &m.read_utility, e.content)
                    .await?;
                read.history.push(ChatEntry {
                    role: if e.role == "user" { Role::User } else { Role::Assistant },
                    content,
                });
            }
        }
        Ok(read)
    }

    /// The message the node took, the one its memories save and compare: the message of the user when the node
    /// takes it, else the message of the node before when it takes that, else none.
    fn taken(&self, node: Entity, input: &NodeInput) -> Option<String> {
        let wants = self.nodes.message().get(node).and_then(|m| m.input).unwrap_or_default();
        if wants.user_message {
            input.user_message.clone()
        } else if wants.agent_message {
            input.agent_message.clone()
        } else {
            None
        }
    }
}

/// The prompt of a summary when the memory does not name one.
const DEFAULT_SYNT_SYSTEM: &str = "You summarize a conversation. Keep the facts, the names, the numbers and the decisions. Write only the summary, in the language of the conversation.";
const DEFAULT_SYNT_USER: &str = "Previous summary:\n{{synt}}\n\nNew exchanges:\n{{chat}}";

impl Program {
    /// Saves the exchange of this turn in every memory of the node, and makes the summary of the
    /// memories that have reached `synt.length` exchanges.
    pub(super) async fn write_memories(
        &self,
        node: Entity,
        input: &NodeInput,
        answer: Option<&Content>,
    ) -> Result<(), Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let Some(external) = self.nodes.external().get(node) else {
            return Ok(());
        };

        let taken = self.taken(node, input);
        for m in &external.memory {
            if !m.write.user && !m.write.assistant {
                continue;
            }
            let memory = self
                .external
                .memory
                .get(&m.id)
                .ok_or_else(|| fail(Rule::UnknownRef, format!("memory `{}` is not built", m.id)))?;
            let MemoryBackend::JsonFile { path } = &memory.backend else {
                return Err(fail(
                    Rule::NotSupported,
                    format!("memory `{}` is kept in a vectordb, which is not supported yet", m.id),
                ));
            };
            let question = taken.clone().filter(|_| m.write.user);
            // what is saved has gone through the utilities of the memory; the vector is of that text
            let question = match question {
                Some(q) => Some(self.text_through(&id, "memory.write", &m.write_utility, q).await?),
                None => None,
            };
            let vector = match (&memory.chat.as_ref().and_then(|c| c.embedding.as_ref()), &question) {
                (Some(embedding), Some(text)) => {
                    let mut vectors = self
                        .embeddings
                        .embed(embedding, std::slice::from_ref(text))
                        .await
                        .map_err(|e| fail(Rule::ModelFailed, e.to_string()).with_detail_if(e.said()))?;
                    vectors.pop()
                }
                _ => None,
            };
            let reply = answer.map(Content::text).filter(|_| m.write.assistant);
            let reply = match reply {
                Some(r) => Some(self.text_through(&id, "memory.write", &m.write_utility, r).await?),
                None => None,
            };
            let saved = self
                .external
                .memory
                .update(path, &id, move |r| {
                    if let Some(content) = question {
                        r.chat.push(Entry {
                            role: "user".into(),
                            content,
                            embedding: vector,
                        });
                    }
                    if let Some(content) = reply {
                        r.chat.push(Entry {
                            role: "assistant".into(),
                            content,
                            embedding: None,
                        });
                    }
                })
                .await
                .map_err(|e| fail(Rule::Io, format!("memory `{}`: {e}", m.id)).with_detail_if(e.detail.as_deref()))?;

            if let Some(synt) = &memory.synt
                && saved.exchanges().len() >= synt.length as usize
            {
                self.summarize(&id, memory, path, saved).await?;
            }
        }
        Ok(())
    }

    /// Writes the new summary from the old one and the exchanges, and takes those exchanges out of `chat`.
    async fn summarize(&self, node: &str, memory: &Memory, path: &str, saved: Record) -> Result<(), Error> {
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(node), rule, msg);
        let Some(synt) = &memory.synt else { return Ok(()) };
        let lines: Vec<String> = saved
            .chat
            .iter()
            .map(|e| format!("{}: {}", e.role, e.content))
            .collect();
        let values = std::collections::HashMap::from([
            (SYNT_PLACEHOLDER.to_string(), saved.synt.clone()),
            (CHAT_PLACEHOLDER.to_string(), lines.join("\n")),
        ]);
        let (system, user) = match synt.prompt.as_deref() {
            Some(pid) => match self.resources.prompts.get(pid).map(|p| &p.body) {
                Some(PromptBody::Content(c)) => (c.system.clone(), c.user.clone()),
                _ => {
                    return Err(fail(
                        Rule::NotSupported,
                        format!("prompt `{pid}` has no text yet (a prompt from a url is not read)"),
                    ));
                }
            },
            None => (DEFAULT_SYNT_SYSTEM.to_string(), DEFAULT_SYNT_USER.to_string()),
        };
        let call = ModelInvocation {
            system: fill(&system, &values).map_err(|e| fail(Rule::PlaceholderUnbound, e))?,
            history: Vec::new(),
            user: fill(&user, &values).map_err(|e| fail(Rule::PlaceholderUnbound, e))?,
            media: Vec::new(),
            tools: Vec::new(),
            output_media: Vec::new(),
            schema: None,
            temperature: None,
            top_p: None,
            max_tokens: Some(synt.max_tokens),
            reasoning: None,
            tool_rounds: Vec::new(),
        };
        let llm = self
            .models
            .get(&synt.model)
            .ok_or_else(|| fail(Rule::UnknownRef, format!("model `{}` is not built", synt.model)))?;
        let response = self.invoke_model(node, &synt.model, llm, &call).await.map_err(|e| {
            fail(Rule::ModelFailed, format!("summary of memory `{}`: {e}", memory.id)).with_detail_if(e.said())
        })?;
        let (stop, thought) = (response.stop_reason, !response.reasoning.is_empty());
        let summary = response.message.filter(|m| !m.trim().is_empty()).ok_or_else(|| {
            fail(
                Rule::ModelFailed,
                format!(
                    "summary of memory `{}`: the model wrote no text (stop reason {stop:?}, it reasoned: {thought})",
                    memory.id
                ),
            )
        })?;

        // the summary is written only over the record it was made from: when another run summarized meanwhile, `synt`
        // or the first messages of `chat` changed, and this summary is dropped; the messages left are summarized at a
        // next write. The messages are compared by role and text: the vector of a message is not part of it
        let before = saved.synt;
        let summarized: Vec<(String, String)> = saved.chat.into_iter().map(|e| (e.role, e.content)).collect();
        self.external
            .memory
            .update(path, node, move |r| {
                let same = r.synt == before
                    && r.chat.len() >= summarized.len()
                    && r.chat
                        .iter()
                        .zip(&summarized)
                        .all(|(e, (role, content))| &e.role == role && &e.content == content);
                if same {
                    r.synt = summary;
                    r.chat.drain(..summarized.len());
                }
            })
            .await
            .map_err(|e| fail(Rule::Io, format!("memory `{}`: {e}", memory.id)).with_detail_if(e.detail.as_deref()))?;
        Ok(())
    }
}
