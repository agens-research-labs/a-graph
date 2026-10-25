//! `external.search`: non-vector retrieval (full-text and SQL engines).

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path};
use crate::network::Destination;
use crate::schema::key;

#[derive(Debug, Clone)]
pub struct Search {
    pub id: String,
    pub provider: String,
    pub url: String,
    /// How to get in; its content is not defined yet.
    pub access: Option<Value>,
}

#[derive(Debug, Default)]
pub struct SearchManager {
    engines: Vec<Search>,
}

impl SearchManager {
    /// Builds from the value of `external.search`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut engines = Vec::new();
        for mut o in Obj::items(&Path::root().key(key::EXTERNAL).key(key::SEARCH), list, &mut errs) {
            let id = o.req::<String>(key::ID, &mut errs);
            let provider = o.req::<String>(key::PROVIDER, &mut errs);
            let url = o.req::<String>(key::URL, &mut errs);
            let access = o.value(key::ACCESS);
            if let (Some(id), Some(provider), Some(url)) = (id, provider, url) {
                engines.push(Search { id, provider, url, access });
            }
        }
        errs.finish(SearchManager { engines })
    }

    /// The urls of the engines, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::EXTERNAL).key(key::SEARCH);
        self.engines.iter().enumerate().map(|(i, s)| Destination::url(base.idx(i).key(key::URL), &s.url)).collect()
    }

    pub fn get(&self, id: &str) -> Option<&Search> {
        self.engines.iter().find(|s| s.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Search> {
        self.engines.iter()
    }
}
