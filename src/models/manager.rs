//! The models of a graph, by id: a node calls a model ONLY by id.

use std::collections::HashMap;

use serde_norway::Value;

use super::llm::Llm;
use super::spec::{self, AzurePublisher, Model, VertexPublisher};
use super::{Anthropic, Bedrock, Gemini, Mistral, Ollama, OpenAi};
use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path};
use crate::network::Destination;
use crate::schema::key;

#[derive(Default)]
pub struct ModelManager {
    models: HashMap<String, Box<dyn Llm>>,
    /// What the yml says about each model: the node needs it to know the family of the media
    /// `provider`, whether `max_tokens` is mandatory, and the host for the allow-list.
    specs: HashMap<String, Model>,
    /// The ids in the order of the yml.
    order: Vec<String>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds from the value of the `models` block, already validated by the Loader. The content
    /// of every model is checked, and everything that can be known without the network: a
    /// missing environment variable behind `{ env: VAR }` is an error here, all of them reported
    /// together. `ambient` credentials are NOT resolved (that happens at the first call).
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut manager = ModelManager::new();
        for o in Obj::items(&Path::root().key(key::MODELS), list, &mut errs) {
            let at = o.at().clone();
            if let Some(model) = spec::read(o, &mut errs) {
                match build_llm(&model, &at) {
                    Ok(llm) => {
                        manager.insert(model.id().to_string(), llm);
                        manager.order.push(model.id().to_string());
                        manager.specs.insert(model.id().to_string(), model);
                    }
                    Err(e) => errs.0.push(e),
                }
            }
        }
        errs.finish(manager)
    }

    /// Registers a model. Ids are unique (the Loader rejects duplicates): a second one with
    /// the same id replaces the first and the replaced model is returned.
    pub fn insert(&mut self, id: impl Into<String>, model: Box<dyn Llm>) -> Option<Box<dyn Llm>> {
        self.models.insert(id.into(), model)
    }

    pub fn get(&self, id: &str) -> Option<&dyn Llm> {
        self.models.get(id).map(|m| m.as_ref())
    }

    /// What the yml says about the model.
    pub fn spec(&self, id: &str) -> Option<&Model> {
        self.specs.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.models.keys().map(String::as_str)
    }

    /// The hosts each model is reached at, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::MODELS);
        self.order
            .iter()
            .enumerate()
            .flat_map(|(i, id)| self.specs[id].destinations(&base.idx(i)))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

fn build_llm(m: &Model, at: &Path) -> Result<Box<dyn Llm>, Error> {
    Ok(match m {
        Model::Anthropic(a) => Box::new(Anthropic::new(a, at)?),
        Model::Openai(o) => Box::new(OpenAi::new(o, at)?),
        Model::Gemini(g) => Box::new(Gemini::new(g, at)?),
        Model::Mistral(x) => Box::new(Mistral::new(x, at)?),
        Model::Bedrock(b) => Box::new(Bedrock::new(b, at)?),
        Model::VertexAi(v) => match v.publisher {
            VertexPublisher::Anthropic => Box::new(Anthropic::vertex(v, at)?),
            VertexPublisher::Google => Box::new(Gemini::vertex(v, at)?),
        },
        Model::AzureFoundry(a) => match a.publisher {
            AzurePublisher::Anthropic => Box::new(Anthropic::azure(a, at)?),
            AzurePublisher::Openai => Box::new(OpenAi::azure(a, at)?),
        },
        Model::Ollama(l) => Box::new(Ollama::ollama(l, at)?),
        Model::Custom(l) => Box::new(Ollama::custom(l, at)?),
    })
}
