//! Where the program calls the utilities of a node: on the data that comes in and on the data that
//! goes out. A chain runs in order, each utility getting what the one before answered.

use super::Program;
use super::node::{Content, NodeInput};
use crate::error::{Error, Path, Phase, Rule};
use crate::logging::{Event, LogLevel};
use crate::models::MediaOutput;
use crate::utilities::{UtilityFailure, UtilityInput, UtilityOutput, call_utility, input_of};

/// The error of a utility that failed in `node`: its message, and what it wrote as the detail.
fn failed(node: &str, why: UtilityFailure) -> Error {
    let e = Error::new(Phase::Execution, Path::node(node), Rule::UtilityFailed, why.message);
    match why.detail {
        Some(detail) => e.with_detail(detail),
        None => e,
    }
}

impl Program {
    /// Runs a text through the utilities `ids`, in order. `place` and `node` tell, in the error and
    /// in the events, where in which node the chain is.
    pub(super) async fn text_through(
        &self,
        node: &str,
        place: &str,
        ids: &[String],
        text: String,
    ) -> Result<String, Error> {
        let fail = |msg: String| Error::new(Phase::Execution, Path::node(node), Rule::UtilityFailed, msg);
        let mut text = text;
        for id in ids {
            let utility = self
                .utilities
                .get(id)
                .ok_or_else(|| fail(format!("utility `{id}` is not built")))?;
            let started = std::time::Instant::now();
            let answer = call_utility(utility, &UtilityInput::Text(text))
                .await
                .map_err(|why| failed(node, why))?;
            self.emit(
                Event::new(LogLevel::Debug, "utility.call")
                    .node(node)
                    .field("id", id.clone())
                    .field("place", place)
                    .field("duration", started.elapsed().as_secs_f64()),
            );
            text = match answer {
                UtilityOutput::Text(t) => t,
                UtilityOutput::File { mime_type, .. } => {
                    return Err(fail(format!(
                        "utility `{id}` answered a file ({mime_type}) where {place} needs a text"
                    )));
                }
            };
        }
        Ok(text)
    }

    /// The incoming text of a node (`message.input.utility`): the messages the node takes go through the
    /// chain, the one it does not take stays as it came. What comes out is the text the node uses everywhere.
    pub(super) async fn transform_input(
        &self,
        node: crate::graph::Entity,
        input: &NodeInput,
    ) -> Result<NodeInput, Error> {
        let message = self.nodes.message().get(node);
        let ids = message.map(|m| m.input_utility.clone()).unwrap_or_default();
        if ids.is_empty() {
            return Ok(input.clone());
        }
        let wants = message.and_then(|m| m.input).unwrap_or_default();
        let name = self.nodes.id(node).unwrap_or_default().to_string();
        let mut out = input.clone();
        if let Some(t) = input.user_message.clone().filter(|_| wants.user_message) {
            out.user_message = Some(self.text_through(&name, "message.input", &ids, t).await?);
        }
        if let Some(t) = input.agent_message.clone().filter(|_| wants.agent_message) {
            out.agent_message = Some(self.text_through(&name, "message.input", &ids, t).await?);
        }
        Ok(out)
    }

    /// The text a node produced (`message.output.utility`), once, before it goes to any destination.
    pub(super) async fn transform_output(
        &self,
        node: crate::graph::Entity,
        content: Option<Content>,
    ) -> Result<Option<Content>, Error> {
        let ids = self
            .nodes
            .message()
            .get(node)
            .and_then(|m| m.output.as_ref())
            .map(|o| o.utility.clone())
            .unwrap_or_default();
        match content {
            Some(c) if !ids.is_empty() => {
                let name = self.nodes.id(node).unwrap_or_default().to_string();
                Ok(Some(Content::Text(
                    self.text_through(&name, "message.output", &ids, c.text()).await?,
                )))
            }
            other => Ok(other),
        }
    }
}

/// What goes through a chain of utilities: a text, a file on disk, or a file in memory (base64).
pub(super) enum Item {
    Text(String),
    Path(String),
    Data {
        mime_type: String,
        data: String,
    },
    /// A media of the graph in base64, with the type the graph gives, if any.
    Base64 {
        mime_type: Option<String>,
        data: String,
    },
}

impl Program {
    /// Runs an item through the utilities `ids`, in order. A file on disk reaches a program by its path and a
    /// service by its bytes, since a service may not see the disk of a-graph; a file in memory reaches every utility
    /// by its bytes. Nothing is written to disk.
    pub(super) async fn item_through(
        &self,
        node: &str,
        place: &str,
        ids: &[String],
        start: Item,
    ) -> Result<Item, Error> {
        let fail = |msg: String| Error::new(Phase::Execution, Path::node(node), Rule::UtilityFailed, msg);
        let mut item = start;
        for id in ids {
            let utility = self
                .utilities
                .get(id)
                .ok_or_else(|| fail(format!("utility `{id}` is not built")))?;
            let input = match &item {
                Item::Text(t) => UtilityInput::Text(t.clone()),
                Item::Path(p) => input_of(utility, p).map_err(|e| {
                    Error::new(
                        Phase::Execution,
                        Path::node(node),
                        Rule::Io,
                        format!("cannot read `{p}`: {e}"),
                    )
                })?,
                Item::Data { mime_type, data } => UtilityInput::Data {
                    mime_type: Some(mime_type.clone()),
                    data: data.clone(),
                },
                Item::Base64 { mime_type, data } => UtilityInput::Data {
                    mime_type: mime_type.clone(),
                    data: data.clone(),
                },
            };
            let started = std::time::Instant::now();
            let answer = call_utility(utility, &input).await.map_err(|why| failed(node, why))?;
            self.emit(
                Event::new(LogLevel::Debug, "utility.call")
                    .node(node)
                    .field("id", id.clone())
                    .field("place", place)
                    .field("duration", started.elapsed().as_secs_f64()),
            );
            item = match answer {
                UtilityOutput::Text(t) => Item::Text(t),
                UtilityOutput::File { mime_type, data } => Item::Data { mime_type, data },
            };
        }
        Ok(item)
    }

    /// A media made by the model goes through its utilities and must come out a file.
    pub(super) async fn media_through(
        &self,
        node: &str,
        ids: &[String],
        media: MediaOutput,
    ) -> Result<MediaOutput, Error> {
        if ids.is_empty() {
            return Ok(media);
        }
        let id = media.id.clone();
        match self
            .item_through(
                node,
                "media.output",
                ids,
                Item::Data {
                    mime_type: media.mime_type,
                    data: media.data,
                },
            )
            .await?
        {
            Item::Data { mime_type, data } => Ok(MediaOutput { mime_type, data, id }),
            _ => Err(Error::new(
                Phase::Execution,
                Path::node(node),
                Rule::UtilityFailed,
                "the utilities of a media must end with a file".to_string(),
            )),
        }
    }
}
