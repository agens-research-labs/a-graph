//! The embedding models of a graph, by id.

use std::collections::HashMap;

use serde::Deserialize;
use serde_norway::Value;

use super::{Embedder, OllamaEmbedding};
use crate::build::{Errs, Obj};
use crate::error::{Errors, Path};
use crate::models::ModelError;
use crate::network::Destination;
use crate::schema::key;

/// The value of `provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Ollama,
}

#[derive(Default)]
pub struct EmbeddingManager {
    embedders: HashMap<String, Box<dyn Embedder>>,
    /// (id, host) in the order of the yml, for the network permission.
    hosts: Vec<(String, String)>,
}

impl EmbeddingManager {
    /// Builds from the value of the `embeddings` block, already validated by the Loader.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut manager = EmbeddingManager::default();
        for mut o in Obj::items(&Path::root().key(key::EMBEDDINGS), list, &mut errs) {
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let provider = o.req::<Provider>(key::PROVIDER, &mut errs);
            let model = o.req::<String>(key::MODEL, &mut errs);
            let host = o.req::<String>(key::HOST, &mut errs);
            if let (Some(id), Some(Provider::Ollama), Some(model), Some(host)) = (id, provider, model, host) {
                match OllamaEmbedding::new(&id, &model, &host, &at) {
                    Ok(e) => {
                        manager.hosts.push((id.clone(), host));
                        manager.embedders.insert(id, Box::new(e));
                    }
                    Err(e) => errs.0.push(e),
                }
            }
        }
        errs.finish(manager)
    }

    pub fn get(&self, id: &str) -> Option<&dyn Embedder> {
        self.embedders.get(id).map(|e| e.as_ref())
    }

    /// Embeds `texts` with the model `id`. A vectordb cites only ids the Loader has checked.
    pub async fn embed(&self, id: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        self.get(id).expect("the Loader checked that the id is declared").embed(texts).await
    }

    /// The host of each embedding model, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::EMBEDDINGS);
        self.hosts.iter().enumerate().map(|(i, (_, host))| Destination::url(base.idx(i).key(key::HOST), host)).collect()
    }

    pub fn len(&self) -> usize {
        self.embedders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.embedders.is_empty()
    }
}
