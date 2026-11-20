//! `external.memory`: the chat memory a node reads and writes, kept outside the process.

use std::collections::HashMap;

use serde::Deserialize;
use serde_norway::Value;

use super::{MemoryFailure, Record, update_record};
use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::resources::{PromptBody, PromptContent, PromptsManager};
use crate::schema::key;

/// The tokens a summary may use when the yml does not say `max_tokens`: a model that reasons
/// spends many of them before it writes.
pub const DEFAULT_SYNT_MAX_TOKENS: u32 = 4000;

/// The placeholders of the prompt of a summary: the previous summary and the exchanges to summarize.
pub const SYNT_PLACEHOLDER: &str = "synt";
pub const CHAT_PLACEHOLDER: &str = "chat";

/// The value of `provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    JsonFile,
    Vectordb,
}

#[derive(Debug, Clone)]
pub enum MemoryBackend {
    JsonFile {
        path: String,
    },
    /// Kept in a vectordb declared in `external.vectordb`.
    Vectordb {
        id: String,
    },
}

/// `synt`: how the memory is summarized.
#[derive(Debug, Clone)]
pub struct SyntConfig {
    /// After how many exchanges the summary is made.
    pub length: u32,
    /// The id of the model that writes the summary.
    pub model: String,
    /// The id of the prompt of the summary; absent = the prompt of the program.
    pub prompt: Option<String>,
    /// The most tokens the summary call may use, reasoning included.
    pub max_tokens: u32,
}

/// `chat`: which exchanges a node gets.
#[derive(Debug, Clone)]
pub struct ChatConfig {
    /// How many exchanges: the last ones, or the most similar with `embedding`.
    pub length: u32,
    /// The id of the `embeddings` entry used to find the similar exchanges.
    pub embedding: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: String,
    pub backend: MemoryBackend,
    pub synt: Option<SyntConfig>,
    pub chat: Option<ChatConfig>,
}

#[derive(Debug, Default)]
pub struct MemoryManager {
    memories: Vec<Memory>,
}

impl MemoryManager {
    /// Builds from the value of `external.memory`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut memories = Vec::new();
        for mut o in Obj::items(&Path::root().key(key::EXTERNAL).key(key::MEMORY), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let provider = o.req::<Provider>(key::PROVIDER, &mut errs);
            let path = o.opt::<String>(key::PATH, &mut errs);
            let vectordb = o.opt::<String>(key::REF, &mut errs);
            let synt = o.sub(key::SYNT, &mut errs).map(|mut s| {
                (
                    s.at().clone(),
                    s.req::<u32>(key::LENGTH, &mut errs),
                    s.req::<String>(key::MODEL, &mut errs),
                    s.opt::<String>(key::PROMPT, &mut errs),
                    s.opt::<u32>(key::MAX_TOKENS, &mut errs),
                )
            });
            let chat = o.sub(key::CHAT, &mut errs).map(|mut c| {
                (
                    c.at().clone(),
                    c.req::<u32>(key::LENGTH, &mut errs),
                    c.opt::<String>(key::EMBEDDING, &mut errs),
                )
            });
            let synt = synt.and_then(|(sat, length, model, prompt, max_tokens)| {
                errs.at_least_one(&sat.key(key::LENGTH), length);
                errs.at_least_one(&sat.key(key::MAX_TOKENS), max_tokens);
                Some(SyntConfig {
                    length: length?,
                    model: model?,
                    prompt,
                    max_tokens: max_tokens.unwrap_or(DEFAULT_SYNT_MAX_TOKENS),
                })
            });
            let chat = chat.and_then(|(cat, length, embedding)| {
                errs.at_least_one(&cat.key(key::LENGTH), length);
                Some(ChatConfig {
                    length: length?,
                    embedding,
                })
            });
            let backend = match provider {
                Some(Provider::JsonFile) => {
                    errs.require(&at, key::PATH, path.is_some(), "provider json_file needs `path`");
                    errs.forbid(&at, key::REF, vectordb.is_some(), "`ref` is for provider vectordb");
                    errs.require(
                        &at,
                        key::CHAT,
                        chat.is_some(),
                        "provider json_file needs `chat`: the node must know how many exchanges to get",
                    );
                    path.map(|path| MemoryBackend::JsonFile { path })
                }
                Some(Provider::Vectordb) => {
                    errs.require(&at, key::REF, vectordb.is_some(), "provider vectordb needs `ref`");
                    errs.forbid(&at, key::PATH, path.is_some(), "`path` is for provider json_file");
                    errs.forbid(&at, key::SYNT, synt.is_some(), "`synt` is for provider json_file");
                    errs.forbid(&at, key::CHAT, chat.is_some(), "`chat` is for provider json_file");
                    vectordb.map(|id| MemoryBackend::Vectordb { id })
                }
                None => None,
            };
            if let (Some(id), Some(backend), true) = (id, backend, errs.0.len() == before) {
                memories.push(Memory {
                    id,
                    backend,
                    synt,
                    chat,
                });
            }
        }
        errs.finish(MemoryManager { memories })
    }

    /// A prompt used for the summary must contain `{{synt}}` and `{{chat}}`. A prompt from a url is checked when the
    /// run has downloaded it (`check_downloaded_prompts`).
    pub fn check_prompts(&self, prompts: &PromptsManager) -> Vec<Error> {
        self.check_prompt_texts(&|pid| match prompts.get(pid).map(|p| &p.body) {
            Some(PromptBody::Content(c)) => Some(c),
            _ => None,
        })
    }

    /// `check_prompts` for the prompts downloaded from their url when the run starts, by id.
    pub fn check_downloaded_prompts(&self, downloaded: &HashMap<String, PromptContent>) -> Vec<Error> {
        self.check_prompt_texts(&|pid| downloaded.get(pid))
    }

    /// The rule of `check_prompts`; `text_of` gives the texts of a prompt by its id, `None` when they are not known.
    fn check_prompt_texts<'a>(&self, text_of: &dyn Fn(&str) -> Option<&'a PromptContent>) -> Vec<Error> {
        let mut errors = Vec::new();
        for (i, m) in self.memories.iter().enumerate() {
            let Some(pid) = m.synt.as_ref().and_then(|s| s.prompt.as_deref()) else {
                continue;
            };
            let at = Path::root()
                .key(key::EXTERNAL)
                .key(key::MEMORY)
                .idx(i)
                .key(key::SYNT)
                .key(key::PROMPT);
            let Some(c) = text_of(pid) else {
                continue;
            };
            let used: Vec<String> = [&c.system, &c.user]
                .into_iter()
                .filter_map(|t| crate::graph::scan(t).ok())
                .flatten()
                .collect();
            for name in [SYNT_PLACEHOLDER, CHAT_PLACEHOLDER] {
                if !used.iter().any(|u| u == name) {
                    errors.push(Error::new(
                        Phase::Interpretation,
                        at.clone(),
                        Rule::PlaceholderUnused,
                        format!(
                            "prompt `{pid}` is the prompt of the summary of memory `{}` and must use {{{{{name}}}}}",
                            m.id
                        ),
                    ));
                }
            }
        }
        errors
    }

    /// Changes the record of `node` in the file at `path`, alone (see `update_record`). The wait for the lock and the
    /// write run on a thread for blocking work, not on a thread of the executor. The lock of the file also keeps out
    /// the other nodes of the same program, since each change opens the lock file again.
    pub async fn update(
        &self,
        path: &str,
        node: &str,
        change: impl FnOnce(&mut Record) + Send + 'static,
    ) -> Result<Record, MemoryFailure> {
        let (path, node) = (path.to_string(), node.to_string());
        tokio::task::spawn_blocking(move || update_record(&path, &node, change))
            .await
            .map_err(|e| MemoryFailure::from(format!("the change of the record stopped: {e}")))?
    }

    pub fn get(&self, id: &str) -> Option<&Memory> {
        self.memories.iter().find(|m| m.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Memory> {
        self.memories.iter()
    }
}
