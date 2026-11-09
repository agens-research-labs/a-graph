//! Where the program calls the utilities of a node: on the data that comes in and on the data that
//! goes out. A chain runs in order, each utility getting what the one before answered.

use super::Program;
use super::node::{Content, NodeInput};
use crate::error::{Error, Path, Phase, Rule};
use crate::logging::{Event, LogLevel};
use crate::models::MediaOutput;
use crate::utilities::{UtilityFailure, UtilityInput, UtilityOutput, call_utility, mime_of};

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

    /// The incoming text of a node (`message.input.utility`): the one from the user and the one from
    /// the previous node go through the chain. What comes out is the text the node uses everywhere.
    pub(super) async fn transform_input(
        &self,
        node: crate::graph::Entity,
        input: &NodeInput,
    ) -> Result<NodeInput, Error> {
        let ids = self
            .nodes
            .message()
            .get(node)
            .map(|m| m.input_utility.clone())
            .unwrap_or_default();
        if ids.is_empty() {
            return Ok(input.clone());
        }
        let name = self.nodes.id(node).unwrap_or_default().to_string();
        let mut out = input.clone();
        if let Some(t) = input.user_message.clone() {
            out.user_message = Some(self.text_through(&name, "message.input", &ids, t).await?);
        }
        if let Some(t) = input.agent_message.clone() {
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
    Data { mime_type: String, data: String },
}

/// The extension a temporary file gets, from its type.
fn extension_of(mime: &str) -> &'static str {
    match mime {
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "video/mp4" => "mp4",
        "text/markdown" => "md",
        "text/html" => "html",
        "application/json" => "json",
        m if m.starts_with("text/") => "txt",
        _ => "bin",
    }
}

impl Program {
    /// Runs an item through the utilities `ids`, in order. A file in memory is written to a temporary
    /// file for the utility, which receives its path, and the file is removed after the call.
    pub(super) async fn item_through(
        &self,
        node: &str,
        place: &str,
        ids: &[String],
        start: Item,
    ) -> Result<Item, Error> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let fail = |msg: String| Error::new(Phase::Execution, Path::node(node), Rule::UtilityFailed, msg);
        let mut item = start;
        for id in ids {
            let utility = self
                .utilities
                .get(id)
                .ok_or_else(|| fail(format!("utility `{id}` is not built")))?;
            let mut temporary: Option<std::path::PathBuf> = None;
            let input = match &item {
                Item::Text(t) => UtilityInput::Text(t.clone()),
                Item::Path(p) => UtilityInput::File {
                    path: p.clone(),
                    mime_type: mime_of(p),
                },
                Item::Data { mime_type, data } => {
                    use base64::Engine;
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(data)
                        .map_err(|e| fail(format!("the file given to utility `{id}` is not valid base64: {e}")))?;
                    let dir = std::env::temp_dir().join(format!("a-graph-{}", std::process::id()));
                    std::fs::create_dir_all(&dir)
                        .map_err(|e| fail(format!("cannot create `{}`: {e}", dir.display())))?;
                    let path = dir.join(format!(
                        "{}.{}",
                        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                        extension_of(mime_type)
                    ));
                    std::fs::write(&path, bytes)
                        .map_err(|e| fail(format!("cannot write `{}`: {e}", path.display())))?;
                    let shown = path.to_string_lossy().to_string();
                    temporary = Some(path);
                    UtilityInput::File {
                        path: shown,
                        mime_type: Some(mime_type.clone()),
                    }
                }
            };
            let started = std::time::Instant::now();
            let answer = call_utility(utility, &input).await;
            if let Some(path) = temporary {
                let _ = std::fs::remove_file(path);
            }
            let answer = answer.map_err(|why| failed(node, why))?;
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
