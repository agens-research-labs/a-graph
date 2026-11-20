//! The urls of the media and of the prompts of a run: downloaded when the run starts, before the first node, only
//! those a node uses (or the summary of a memory a node uses), one at a time, and kept until the run ends. Building
//! the program never calls them.
//!
//! All the urls of a run together may weigh `BUDGET` and take `TIME`: a set of media sent to the service inside the
//! graph, in `base64`, is bounded by its body of 64 MiB, and a url may not hold more than what it replaces. The time
//! is the one of such a body at 10 Mbit/s.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use super::Program;
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::nodes::component::InFrom;
use crate::logging::{Event, LogLevel};
use crate::resources::{MediaKind, Origin, PromptBody, PromptContent, Source};
use crate::schema::key;
use crate::yaml::{Found, MAX_YML_EVENTS, first_refused};

/// The most bytes all the urls of a run may weigh together.
const BUDGET: u64 = 64 * 1024 * 1024;

/// The longest all the downloads of a run may take together.
const TIME: Duration = Duration::from_secs(60);

/// What the urls of the run in progress answered: the bytes of each media and the texts of each prompt, by id.
#[derive(Debug, Default)]
pub(super) struct Downloaded {
    /// Each body as it was read: an `Arc` of the `Vec` itself, so that keeping it copies nothing.
    media: HashMap<String, Arc<Vec<u8>>>,
    prompts: HashMap<String, PromptContent>,
}

/// One resource to download.
struct Wanted<'a> {
    what: What,
    id: &'a str,
    /// Where its `source` is in the yml: the errors are placed on its `url`.
    at: Path,
    url: &'a str,
    headers: &'a BTreeMap<String, String>,
}

enum What {
    /// `text`: a `md` that some node reads without utilities, so it must be text.
    Media {
        text: bool,
    },
    Prompt,
}

impl Program {
    /// Downloads the urls the nodes use, checks what they answered and keeps it for the run. The first failure stops
    /// the run, before any node.
    pub(super) async fn download_sources(&self) -> Result<(), Error> {
        let wanted = self.wanted();
        if wanted.is_empty() {
            return Ok(());
        }
        let client = crate::net::client().build().map_err(|e| {
            Error::new(
                Phase::Execution,
                Path::root().key(key::RESOURCES),
                Rule::Io,
                format!("cannot build the HTTP client: {e}"),
            )
        })?;
        let deadline = tokio::time::Instant::now() + TIME;
        let mut left = BUDGET;
        let mut got = Downloaded::default();
        for w in wanted {
            let fail = |rule: Rule, why: String| {
                let name = match w.what {
                    What::Media { .. } => "media",
                    What::Prompt => "prompt",
                };
                Error::new(
                    Phase::Execution,
                    w.at.key(key::URL),
                    rule,
                    format!("{name} `{}`: {why}", w.id),
                )
            };
            let began = std::time::Instant::now();
            let bytes = match tokio::time::timeout_at(deadline, fetch(&client, w.url, w.headers, left)).await {
                Ok(Ok(b)) => b,
                Ok(Err(why)) => return Err(fail(Rule::Io, why)),
                Err(_) => {
                    return Err(fail(
                        Rule::Io,
                        format!("the downloads of the run passed {} seconds", TIME.as_secs()),
                    ));
                }
            };
            left -= bytes.len() as u64;
            let code = match w.what {
                What::Media { .. } => "read.media",
                What::Prompt => "read.prompt",
            };
            self.emit(
                self.traced(
                    Event::new(LogLevel::Debug, code)
                        .field("id", w.id)
                        .field("bytes", bytes.len())
                        .field("duration", began.elapsed().as_secs_f64()),
                ),
            );
            match w.what {
                What::Media { text } => {
                    if text && let Err(e) = std::str::from_utf8(&bytes) {
                        return Err(fail(Rule::Io, format!("it is not text: {e}")));
                    }
                    got.media.insert(w.id.to_string(), Arc::new(bytes));
                }
                What::Prompt => {
                    let content = prompt_of(&bytes).map_err(|(rule, why)| fail(rule, why))?;
                    got.prompts.insert(w.id.to_string(), content);
                }
            }
        }
        // the placeholders of the prompts downloaded, as the check of the file does for the others
        let mut broken = crate::graph::downloaded_placeholders(&self.nodes, &self.external, &got.prompts);
        broken.extend(self.external.memory.check_downloaded_prompts(&got.prompts));
        // a run ends with one error: the first, as `connect_mcp` does (the check of the file reports them all)
        if let Some(first) = broken.into_iter().next() {
            return Err(first);
        }
        *self.sources.lock().unwrap() = Some(Arc::new(got));
        Ok(())
    }

    /// Lets go of what the urls of the run that ends answered.
    pub(super) fn close_sources(&self) {
        self.sources.lock().unwrap().take();
    }

    /// The bytes the url of media `id` answered in the run in progress.
    pub(super) fn downloaded_media(&self, id: &str) -> Option<Arc<Vec<u8>>> {
        self.sources.lock().unwrap().as_ref()?.media.get(id).cloned()
    }

    /// The texts of prompt `id`: written in the graph, read from its file, or what its url answered when the run
    /// started.
    pub(super) fn prompt_texts(&self, id: &str) -> Result<PromptContent, String> {
        match self.resources.prompts.get(id).map(|p| &p.body) {
            Some(PromptBody::Content(c)) => Ok(c.clone()),
            Some(PromptBody::Source(_)) => self
                .sources
                .lock()
                .unwrap()
                .as_ref()
                .and_then(|d| d.prompts.get(id).cloned())
                .ok_or_else(|| format!("prompt `{id}`: its url was not downloaded, the run has not started")),
            None => Err(format!("prompt `{id}` is not declared")),
        }
    }

    /// The media and the prompts at a url that a node uses, or the summary of a memory a node uses, each once, in the
    /// order of the yml: the media first, then the prompts.
    fn wanted(&self) -> Vec<Wanted<'_>> {
        let mut media_used: HashSet<&str> = HashSet::new();
        // a `md` some node reads as it is, with no utility in between
        let mut read_as_text: HashSet<&str> = HashSet::new();
        let mut prompts_used: HashSet<&str> = HashSet::new();
        for e in self.nodes.entities() {
            if let Some(m) = self.nodes.media().get(e) {
                for entry in m.input.iter().filter(|i| i.from == InFrom::Resources) {
                    media_used.insert(&entry.id);
                    if entry.utility.is_empty() {
                        read_as_text.insert(&entry.id);
                    }
                }
            }
            if let Some(p) = self.nodes.resources().get(e).and_then(|r| r.prompt.as_deref()) {
                prompts_used.insert(p);
            }
            for used in self
                .nodes
                .external()
                .get(e)
                .map(|x| x.memory.as_slice())
                .unwrap_or_default()
            {
                if let Some(p) = self
                    .external
                    .memory
                    .get(&used.id)
                    .and_then(|m| m.synt.as_ref())
                    .and_then(|s| s.prompt.as_deref())
                {
                    prompts_used.insert(p);
                }
            }
        }
        let mut wanted = Vec::new();
        let media_base = Path::root().key(key::RESOURCES).key(key::MEDIA);
        for (i, m) in self.resources.media.iter().enumerate() {
            if let (Origin::Source(Source::Url { url, headers, .. }), true) =
                (&m.origin, media_used.contains(m.id.as_str()))
            {
                wanted.push(Wanted {
                    what: What::Media {
                        text: m.kind == MediaKind::Md && read_as_text.contains(m.id.as_str()),
                    },
                    id: &m.id,
                    at: media_base.idx(i).key(key::SOURCE),
                    url,
                    headers,
                });
            }
        }
        let prompts_base = Path::root().key(key::RESOURCES).key(key::PROMPTS);
        for (i, p) in self.resources.prompts.iter().enumerate() {
            if let (PromptBody::Source(Source::Url { url, headers, .. }), true) =
                (&p.body, prompts_used.contains(p.id.as_str()))
            {
                wanted.push(Wanted {
                    what: What::Prompt,
                    id: &p.id,
                    at: prompts_base.idx(i).key(key::SOURCE),
                    url,
                    headers,
                });
            }
        }
        wanted
    }
}

/// The body `url` answers, if it is at most `left` bytes. The url is never part of what is told: its query may hold a
/// secret.
async fn fetch(
    client: &reqwest::Client,
    url: &str,
    headers: &BTreeMap<String, String>,
    left: u64,
) -> Result<Vec<u8>, String> {
    let over = || format!("the urls of the run passed {} MiB together", BUDGET / 1024 / 1024);
    let scheme = url::Url::parse(url).map(|u| u.scheme().to_string()).unwrap_or_default();
    if scheme != "http" && scheme != "https" {
        return Err(format!("the scheme `{scheme}` is not http or https"));
    }
    let mut request = client.get(url);
    for (name, var) in headers {
        let value = std::env::var(var).map_err(|_| format!("environment variable `{var}` is not set"))?;
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| format!("`{name}` is not a valid header name"))?;
        let value =
            crate::net::secret_value(&value).map_err(|_| format!("the value of the header `{name}` is not valid"))?;
        request = request.header(name, value);
    }
    let mut response = request.send().await.map_err(|e| e.without_url().to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("the server answered {}", status.as_u16()));
    }
    if response.content_length().is_some_and(|n| n > left) {
        return Err(over());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.without_url().to_string())? {
        if (body.len() + chunk.len()) as u64 > left {
            return Err(over());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// The texts of a prompt from what its url answered: a yml with `content: { system, user }`, as a prompt file, with no
/// alias, no tag and at most `MAX_YML_EVENTS` events, since it comes from outside the graph.
fn prompt_of(bytes: &[u8]) -> Result<PromptContent, (Rule, String)> {
    let text = std::str::from_utf8(bytes).map_err(|e| (Rule::BadValue, format!("it is not text: {e}")))?;
    let refused = |why: &str| (Rule::NotSupported, why.to_string());
    match first_refused(text) {
        Some(Found::Alias) => {
            return Err(refused(
                "its text has a YAML alias, which is not accepted: write the value again where it is used",
            ));
        }
        Some(Found::Tag) => return Err(refused("its text has a YAML tag (`!name`), which is not accepted")),
        Some(Found::TooManyEvents) => {
            return Err((
                Rule::NotSupported,
                format!("its text has more than {MAX_YML_EVENTS} YAML events (values, keys, lists and maps)"),
            ));
        }
        None => {}
    }
    crate::resources::parse_content(text, "the text").map_err(|why| {
        (
            Rule::BadValue,
            format!("its text must hold `content: {{ system, user }}`: {why}"),
        )
    })
}
