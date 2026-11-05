//! The managers of the `external` block: connections to services outside the process.
//! They build and check the configuration; talking to the services is not defined yet.

mod memory;
mod memory_file;
mod read_vectordb;
mod search;
mod vectordb;

pub use memory::*;
pub use memory_file::*;
pub use search::*;
pub use vectordb::*;

use serde_norway::Value;

use crate::error::Errors;
use crate::network::Destination;
use crate::schema::key;
use crate::secrets::Secret;

/// The three managers, built from the value of the `external` block.
#[derive(Debug, Default)]
pub struct External {
    pub vectordb: VectordbManager,
    pub search: SearchManager,
    pub memory: MemoryManager,
}

impl External {
    /// Every network destination of `external`, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        [self.vectordb.destinations(), self.search.destinations()].concat()
    }

    /// The environment variables of the external services and where their values go, for
    /// `allowed_env`: only the vector databases have one.
    pub fn secrets(&self) -> Vec<Secret> {
        self.vectordb.secrets()
    }

    /// Builds all three. On failure returns the errors of every manager, not only the first.
    pub fn build(block: Option<Value>) -> Result<Self, Errors> {
        let mut block = match block {
            Some(Value::Mapping(m)) => m,
            _ => Default::default(),
        };
        let vectordb = VectordbManager::build(block.remove(key::VECTORDB));
        let search = SearchManager::build(block.remove(key::SEARCH));
        let memory = MemoryManager::build(block.remove(key::MEMORY));

        match (vectordb, search, memory) {
            (Ok(vectordb), Ok(search), Ok(memory)) => Ok(External {
                vectordb,
                search,
                memory,
            }),
            (a, b, c) => {
                let mut all = Vec::new();
                for r in [a.err(), b.err(), c.err()].into_iter().flatten() {
                    all.extend(r.0);
                }
                Err(Errors(all))
            }
        }
    }
}
