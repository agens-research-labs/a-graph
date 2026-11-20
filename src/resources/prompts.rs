//! `resources.prompts`.

use serde_norway::Value;

use super::common::{Errs, Obj, list_path};
use super::source::{self, Source};
use crate::error::{Error, Errors, Path, Rule};
use crate::network::Destination;
use crate::schema::key;
use crate::secrets::Secret;

#[derive(Debug, Clone)]
pub struct PromptContent {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone)]
pub enum PromptBody {
    /// Written in the yml, or read from the file of `source.path`.
    Content(PromptContent),
    /// A source read when the run starts (`url`).
    Source(Source),
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub id: String,
    pub body: PromptBody,
}

#[derive(Debug, Default)]
pub struct PromptsManager {
    prompts: Vec<Prompt>,
}

impl PromptsManager {
    /// Builds from the value of `resources.prompts`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut prompts = Vec::new();
        for mut o in Obj::items(&list_path(key::PROMPTS), list, &mut errs) {
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let (has_content, has_source) = (o.has(key::CONTENT), o.has(key::SOURCE));
            let content = o.sub(key::CONTENT, &mut errs).and_then(|c| content(c, &mut errs));
            let src = o.sub(key::SOURCE, &mut errs);
            let body = match (has_content, has_source) {
                (true, false) => content.map(PromptBody::Content),
                (false, true) => {
                    let at = at.key(key::SOURCE);
                    match src.and_then(|s| source::build(s, false, &mut errs)) {
                        Some(Source::Path(path)) => {
                            read_file(&at.key(key::PATH), &path, &mut errs).map(PromptBody::Content)
                        }
                        other => other.map(PromptBody::Source),
                    }
                }
                _ => {
                    errs.add(&at, Rule::ExactlyOne, "a prompt needs exactly one of content, source");
                    None
                }
            };
            if let (Some(id), Some(body)) = (id, body) {
                prompts.push(Prompt { id, body });
            }
        }
        errs.finish(PromptsManager { prompts })
    }

    /// The urls prompts are read from, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = list_path(key::PROMPTS);
        self.prompts
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match &p.body {
                PromptBody::Source(Source::Url { url, .. }) => {
                    Some(Destination::url(base.idx(i).key(key::SOURCE).key(key::URL), url))
                }
                _ => None,
            })
            .collect()
    }

    /// The headers of the prompts read from a url and the url each goes to, for `allowed_env`.
    pub fn secrets(&self) -> Vec<Secret> {
        let base = list_path(key::PROMPTS);
        self.prompts
            .iter()
            .enumerate()
            .flat_map(|(i, p)| match &p.body {
                PromptBody::Source(s) => source::secrets(s, &base.idx(i).key(key::SOURCE)),
                PromptBody::Content(_) => vec![],
            })
            .collect()
    }

    /// A header whose environment variable is not set, for every prompt read from a url.
    pub fn missing_env(&self) -> Vec<Error> {
        let base = list_path(key::PROMPTS);
        let mut errs = Errs::default();
        for (i, p) in self.prompts.iter().enumerate() {
            if let PromptBody::Source(s) = &p.body {
                source::missing_env(s, &base.idx(i).key(key::SOURCE), &mut errs);
            }
        }
        errs.0
    }

    pub fn get(&self, id: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Prompt> {
        self.prompts.iter()
    }
}

/// `{ system, user }`.
fn content(mut c: Obj, errs: &mut Errs) -> Option<PromptContent> {
    let system = c.req::<String>(key::SYSTEM, errs);
    let user = c.req::<String>(key::USER, errs);
    Some(PromptContent {
        system: system?,
        user: user?,
    })
}

/// Opens the file as written in the yml and reads the `content` of the prompt from it.
fn read_file(at: &Path, path: &str, errs: &mut Errs) -> Option<PromptContent> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            errs.add(at, Rule::Io, format!("cannot read `{path}`: {e}"));
            return None;
        }
    };
    // whatever is wrong inside the file is one error, placed on the `path` that points to it
    match parse_content(&text, "the file") {
        Ok(c) => Some(c),
        Err(why) => {
            errs.add(
                at,
                Rule::BadValue,
                format!("`{path}` must hold `content: {{ system, user }}`: {why}"),
            );
            None
        }
    }
}

/// The `content: { system, user }` of the text of a prompt file, or why the text is not one: a file of `path`, or
/// the answer of a `url`. `what` names it in the reason: `the file`, `the text`.
pub(crate) fn parse_content(text: &str, what: &str) -> Result<PromptContent, String> {
    let mut inner = Errs::default();
    let parsed = serde_norway::from_str::<Value>(text)
        .map_err(|e| e.to_string())
        .and_then(|v| Obj::new(v, Path::root(), &mut inner).ok_or_else(|| format!("{what} must be a map")))
        .map(|mut file| {
            file.req_sub(key::CONTENT, &mut inner)
                .and_then(|c| content(c, &mut inner))
        });
    match (parsed, inner.0.is_empty()) {
        (Ok(Some(c)), true) => Ok(c),
        (Err(why), _) => Err(why),
        (_, _) => Err(inner.0.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("; ")),
    }
}
