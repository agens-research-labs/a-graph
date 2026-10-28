//! `external.vectordb`: vector databases outside the process.

use serde::Deserialize;
use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::network::Destination;
use crate::schema::key;

/// The tenant and the database that a Chroma server has when nobody created others.
pub const DEFAULT_CHROMA_TENANT: &str = "default_tenant";
pub const DEFAULT_CHROMA_DATABASE: &str = "default_database";

/// The value of `provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Qdrant,
    Chromadb,
}

#[derive(Debug, Clone)]
pub enum VectorBackend {
    Qdrant { url: String, collection: String, text_field: String },
    Chromadb { url: String, collection: String, tenant: String, database: String },
}

#[derive(Debug, Clone)]
pub struct Vectordb {
    pub id: String,
    pub backend: VectorBackend,
    /// The name of the environment variable holding the key, never the key.
    pub api_key: Option<String>,
    pub collection: Option<String>,
    pub top_k: Option<u32>,
    /// Seconds one search may take; absent = the default of the reader.
    pub timeout: Option<u32>,
    /// The id of the `embeddings` entry that embeds the query.
    pub embedding: String,
}

#[derive(Debug, Default)]
pub struct VectordbManager {
    databases: Vec<Vectordb>,
    pub(super) client: reqwest::Client,
}

impl VectordbManager {
    /// Builds from the value of `external.vectordb`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut databases = Vec::new();
        for mut o in Obj::items(&Path::root().key(key::EXTERNAL).key(key::VECTORDB), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let provider = o.req::<Provider>(key::PROVIDER, &mut errs);
            let url = o.opt::<String>(key::URL, &mut errs);
            let api_key = o.opt_secret(key::API_KEY, &mut errs);
            let collection = o.opt::<String>(key::COLLECTION, &mut errs);
            let top_k = o.opt::<u32>(key::TOP_K, &mut errs);
            let timeout = o.opt::<u32>(key::TIMEOUT, &mut errs);
            if timeout == Some(0) {
                errs.add(&at.key(key::TIMEOUT), Rule::BadValue, "`timeout` is at least 1 second");
            }
            let embedding = o.req::<String>(key::EMBEDDING, &mut errs);

            let text_field = o.opt::<String>(key::TEXT_FIELD, &mut errs);
            match provider {
                Some(Provider::Qdrant) => errs.require(&at, key::TEXT_FIELD, text_field.is_some(), "provider qdrant needs `text_field`"),
                Some(Provider::Chromadb) => errs.forbid(&at, key::TEXT_FIELD, text_field.is_some(), "chromadb keeps the text in its own documents"),
                None => {}
            }
            let tenant = o.opt::<String>(key::TENANT, &mut errs);
            let database = o.opt::<String>(key::DATABASE, &mut errs);
            if provider == Some(Provider::Qdrant) {
                errs.forbid(&at, key::TENANT, tenant.is_some(), "tenant is for provider chromadb");
                errs.forbid(&at, key::DATABASE, database.is_some(), "database is for provider chromadb");
            }
            let name = match provider {
                Some(Provider::Qdrant) => "qdrant",
                _ => "chromadb",
            };
            errs.require(&at, key::URL, url.is_some(), &format!("provider {name} needs `url`"));
            errs.require(&at, key::COLLECTION, collection.is_some(), &format!("provider {name} needs `collection`"));
            let backend = match (provider, url, collection.clone()) {
                (Some(Provider::Qdrant), Some(url), Some(collection)) => text_field.map(|text_field| VectorBackend::Qdrant { url, collection, text_field }),
                (Some(Provider::Chromadb), Some(url), Some(collection)) => Some(VectorBackend::Chromadb {
                    url,
                    collection,
                    tenant: tenant.unwrap_or_else(|| DEFAULT_CHROMA_TENANT.to_string()),
                    database: database.unwrap_or_else(|| DEFAULT_CHROMA_DATABASE.to_string()),
                }),
                _ => None,
            };
            if let (Some(id), Some(backend), Some(embedding), true) = (id, backend, embedding, errs.0.len() == before) {
                databases.push(Vectordb { id, backend, api_key, collection, top_k, timeout, embedding });
            }
        }
        match crate::net::client().build() {
            Ok(client) => errs.finish(VectordbManager { databases, client }),
            Err(e) => {
                errs.add(&Path::root().key(key::EXTERNAL).key(key::VECTORDB), Rule::Io, format!("cannot build the HTTP client: {e}"));
                Err(Errors(errs.0))
            }
        }
    }

    /// The urls of the vector databases, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::EXTERNAL).key(key::VECTORDB);
        self.databases
            .iter()
            .enumerate()
            .map(|(i, d)| match &d.backend {
                VectorBackend::Qdrant { url, .. } | VectorBackend::Chromadb { url, .. } => Destination::url(base.idx(i).key(key::URL), url),
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Vectordb> {
        self.databases.iter().find(|d| d.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Vectordb> {
        self.databases.iter()
    }
}
