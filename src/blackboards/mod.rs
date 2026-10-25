//! The manager of the `blackboards` block: shared state between agents, always Markdown.
//! A node never touches a board: it reads, writes and cleans through this manager.

mod header;

use std::sync::Mutex;

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
    /// A file, opened as written in the yml. The lock serializes read, change and write.
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
                let content = std::fs::read_to_string(path).map_err(|e| io(b, path, e))?;
                std::fs::write(path, header::write(&content, text, place)).map_err(|e| io(b, path, e))
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
                let content = std::fs::read_to_string(path).map_err(|e| io(b, path, e))?;
                std::fs::write(path, header::cleared(&content)).map_err(|e| io(b, path, e))
            }
            Store::Url { url } => Err(not_read(b, url)),
        }
    }

    fn board(&self, id: &str) -> Result<&Board, Error> {
        self.boards.iter().find(|b| b.id == id).ok_or_else(|| {
            Error::new(Phase::Execution, Path::root().key(key::BLACKBOARDS), Rule::UnknownRef, format!("blackboard `{id}` is not declared"))
        })
    }
}

fn at(b: &Board) -> Path {
    Path::root().key(key::BLACKBOARDS).idx(b.index)
}

fn io(b: &Board, path: &str, e: std::io::Error) -> Error {
    Error::new(Phase::Execution, at(b), Rule::Io, format!("blackboard `{}`: `{path}`: {e}", b.id))
}

fn not_read(b: &Board, url: &str) -> Error {
    Error::new(Phase::Execution, at(b), Rule::NotSupported, format!("blackboard `{}`: the board at `{url}` cannot be used, a url source is not supported yet", b.id))
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
        (Some(path), None) => check_file(&at.key(key::FILE), &path, errs).then(|| Store::File { path, lock: Mutex::new(()) }),
        // `access` is accepted; its content is not defined yet, so it is not kept.
        (None, Some(url)) => Some(Store::Url { url }),
        _ => {
            errs.add(at, Rule::ExactlyOne, "a source is `memory`, or `file`, or `url`");
            None
        }
    }
}
