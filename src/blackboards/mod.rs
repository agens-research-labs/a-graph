//! The manager of the `blackboards` block: shared state between agents, always Markdown.
//! A node never touches a board: it reads, writes and cleans through this manager.

mod header;

use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::network::Destination;
use crate::schema::key;

/// Where the text is added to the board, below the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Place {
    Top,
    Bottom,
}

/// Where the Markdown of a board lives.
#[derive(Debug)]
enum Store {
    /// In the process: starts empty, lost when the program ends.
    Memory(Mutex<String>),
    /// A file, opened as written in the yml. The mutex serializes read, change and write inside the
    /// program; the lock of `<file>.lock` does it between programs and processes (`change_file`).
    File { path: String, lock: Mutex<()> },
    /// Not read yet.
    Url { url: String },
}

#[derive(Debug)]
struct Board {
    id: String,
    /// Position in the yml list, for the path of an error.
    index: usize,
    store: Store,
}

#[derive(Debug, Default)]
pub struct BlackboardManager {
    boards: Vec<Board>,
}

impl BlackboardManager {
    /// Builds from the value of `blackboards`. A board in a file must exist and start with the
    /// mandatory header.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut boards = Vec::new();
        for mut o in Obj::items(&Path::root().key(key::BLACKBOARDS), list, &mut errs) {
            let index = boards.len();
            let at = o.at().key(key::SOURCE);
            let id = o.req::<String>(key::ID, &mut errs);
            let store = o.value(key::SOURCE).and_then(|v| store(v, &at, &mut errs));
            if let (Some(id), Some(store)) = (id, store) {
                boards.push(Board { id, index, store });
            }
        }
        errs.finish(BlackboardManager { boards })
    }

    /// The urls of the boards that live at an address, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::BLACKBOARDS);
        self.boards
            .iter()
            .filter_map(|b| match &b.store {
                Store::Url { url } => Some(Destination::url(base.idx(b.index).key(key::SOURCE).key(key::URL), url)),
                _ => None,
            })
            .collect()
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.boards.iter().map(|b| b.id.as_str())
    }

    /// The whole Markdown of the board, header included.
    pub fn read(&self, id: &str) -> Result<String, Error> {
        let b = self.board(id)?;
        match &b.store {
            Store::Memory(m) => Ok(m.lock().unwrap().clone()),
            Store::File { path, lock } => {
                let _held = lock.lock().unwrap();
                std::fs::read_to_string(path).map_err(|e| io(b, path, e))
            }
            Store::Url { url } => Err(not_read(b, url)),
        }
    }

    /// Adds `text` below the header, at the top of the body or at the bottom.
    pub fn write(&self, id: &str, text: &str, place: Place) -> Result<(), Error> {
        let b = self.board(id)?;
        match &b.store {
            Store::Memory(m) => {
                let mut content = m.lock().unwrap();
                *content = header::write(&content, text, place);
                Ok(())
            }
            Store::File { path, lock } => {
                let _held = lock.lock().unwrap();
                change_file(b, path, |content| header::write(content, text, place))
            }
            Store::Url { url } => Err(not_read(b, url)),
        }
    }

    /// Removes everything below the header; the header stays.
    pub fn cleanup(&self, id: &str) -> Result<(), Error> {
        let b = self.board(id)?;
        match &b.store {
            Store::Memory(m) => {
                let mut content = m.lock().unwrap();
                *content = header::cleared(&content);
                Ok(())
            }
            Store::File { path, lock } => {
                let _held = lock.lock().unwrap();
                change_file(b, path, header::cleared)
            }
            Store::Url { url } => Err(not_read(b, url)),
        }
    }

    fn board(&self, id: &str) -> Result<&Board, Error> {
        self.boards.iter().find(|b| b.id == id).ok_or_else(|| {
            Error::new(
                Phase::Execution,
                Path::root().key(key::BLACKBOARDS),
                Rule::UnknownRef,
                format!("blackboard `{id}` is not declared"),
            )
        })
    }
}

fn at(b: &Board) -> Path {
    Path::root().key(key::BLACKBOARDS).idx(b.index)
}

fn io(b: &Board, path: &str, e: std::io::Error) -> Error {
    Error::new(
        Phase::Execution,
        at(b),
        Rule::Io,
        format!("blackboard `{}`: `{path}`: {e}", b.id),
    )
}

/// Reads the file of `b`, changes it and writes it back, holding an exclusive lock on `<path>.lock` from
/// the read to the write: the other programs and processes that change the same file wait. The lock file
/// is never deleted, since deleting it while another run waits on it would let two runs hold the lock. A
/// file whose header is no longer valid is not changed: without its header, the whole file would be
/// taken as the body and written back without it.
fn change_file(b: &Board, path: &str, change: impl FnOnce(&str) -> String) -> Result<(), Error> {
    let _held = lock_file(b, path)?;
    let content = std::fs::read_to_string(path).map_err(|e| io(b, path, e))?;
    header::header_end(&content).map_err(|why| {
        Error::new(
            Phase::Execution,
            at(b),
            Rule::BlackboardHeader,
            format!("blackboard `{}`: `{path}`: {why}", b.id),
        )
    })?;
    replace_file(b, path, &change(&content))
}

/// An exclusive lock on `<path>.lock`, held until the file is dropped.
fn lock_file(b: &Board, path: &str) -> Result<std::fs::File, Error> {
    let name = format!("{path}.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&name)
        .map_err(|e| io(b, &name, e))?;
    file.lock().map_err(|e| io(b, &name, e))?;
    Ok(file)
}

/// Writes `content` to `path` in one step: a temporary file of its own, then renamed, so a reader sees
/// the file before or after the write, never half of it.
fn replace_file(b: &Board, path: &str, content: &str) -> Result<(), Error> {
    static WRITES: AtomicU64 = AtomicU64::new(0);
    let temporary = format!(
        "{path}.{}.{}.tmp",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::Relaxed)
    );
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| io(b, &temporary, e))
        .and_then(|()| std::fs::rename(&temporary, path).map_err(|e| io(b, path, e)));
    if written.is_err() {
        // The error of the write is the one that matters; the file may not even exist.
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

fn not_read(b: &Board, url: &str) -> Error {
    Error::new(
        Phase::Execution,
        at(b),
        Rule::NotSupported,
        format!(
            "blackboard `{}`: the board at `{url}` cannot be used, a url source is not supported yet",
            b.id
        ),
    )
}

/// The file exists and starts with the header.
fn check_file(at: &Path, file: &str, errs: &mut Errs) -> bool {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            errs.add(at, Rule::Io, format!("cannot read `{file}`: {e}"));
            return false;
        }
    };
    match header::header_end(&text) {
        Ok(_) => true,
        Err(why) => {
            errs.add(at, Rule::BlackboardHeader, format!("`{file}`: {why}"));
            false
        }
    }
}

/// `memory`, `{ file }` or `{ url, access }`.
fn store(v: Value, at: &Path, errs: &mut Errs) -> Option<Store> {
    if v.as_str() == Some("memory") {
        return Some(Store::Memory(Mutex::new(String::new())));
    }
    let mut o = Obj::new(v, at.clone(), errs)?;
    match (o.opt::<String>(key::FILE, errs), o.opt::<String>(key::URL, errs)) {
        (Some(path), None) => check_file(&at.key(key::FILE), &path, errs).then(|| Store::File {
            path,
            lock: Mutex::new(()),
        }),
        // `access` is accepted; its content is not defined yet, so it is not kept.
        (None, Some(url)) => Some(Store::Url { url }),
        _ => {
            errs.add(at, Rule::ExactlyOne, "a source is `memory`, or `file`, or `url`");
            None
        }
    }
}
