//! The nodes of the graph as entities and components. An entity is only the id of a node; each
//! kind of component has its own store, indexed by entity. The set of components is closed.

pub mod component;
mod read;

pub(crate) use read::structured_output;

use std::collections::HashMap;

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::schema::key;
use component::*;

/// A node: its position in `graph.nodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// One slot per entity: present when the node has that component.
#[derive(Debug)]
pub struct Components<T> {
    slots: Vec<Option<T>>,
}

impl<T> Components<T> {
    fn new() -> Self {
        Components { slots: Vec::new() }
    }

    fn push(&mut self, value: Option<T>) {
        self.slots.push(value);
    }

    pub fn get(&self, e: Entity) -> Option<&T> {
        self.slots.get(e.0 as usize).and_then(Option::as_ref)
    }

    /// The entities that have the component.
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|v| (Entity(i as u32), v)))
    }
}

#[derive(Debug)]
pub struct Nodes {
    ids: Vec<String>,
    by_id: HashMap<String, Entity>,
    model: Components<Model>,
    external: Components<External>,
    resources: Components<Resources>,
    blackboard: Components<Blackboard>,
    media: Components<Media>,
    message: Components<Message>,
    validation: Components<Validation>,
}

impl Nodes {
    /// Builds from the value of `graph.nodes`, already validated by the Loader.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut nodes = Nodes {
            ids: Vec::new(),
            by_id: HashMap::new(),
            model: Components::new(),
            external: Components::new(),
            resources: Components::new(),
            blackboard: Components::new(),
            media: Components::new(),
            message: Components::new(),
            validation: Components::new(),
        };
        for mut o in Obj::items(&Path::root().key(key::GRAPH).key(key::NODES), list, &mut errs) {
            let at = o.at().clone();
            let Some(id) = o.req::<String>(key::ID, &mut errs) else {
                continue;
            };
            // the entity of this node is the next position
            let entity = Entity(nodes.ids.len() as u32);
            if nodes.by_id.insert(id.clone(), entity).is_some() {
                errs.add(
                    &at.key(key::ID),
                    Rule::DuplicateId,
                    format!("node id `{id}` is already used"),
                );
            }
            nodes.ids.push(id);
            // a validation node is made of `id` and `validation-node`
            if o.has(key::VALIDATION_NODE) {
                for other in [
                    key::MODEL,
                    key::EXTERNAL,
                    key::RESOURCES,
                    key::BLACKBOARD,
                    key::MEDIA,
                    key::MESSAGE,
                ] {
                    errs.forbid(
                        &at,
                        other,
                        o.has(other),
                        "a validation node has only `id` and `validation-node`",
                    );
                }
            }
            // a validation node has the same blocks inside `validation-node`: they are read as a
            // node's, and the node also carries `Validation`
            let judge = o.sub(key::VALIDATION_NODE, &mut errs);
            let is_judge = judge.is_some();
            let mut src = judge.unwrap_or(o);
            nodes
                .model
                .push(src.sub(key::MODEL, &mut errs).and_then(|m| read::model(m, &mut errs)));
            nodes.external.push(
                src.sub(key::EXTERNAL, &mut errs)
                    .map(|e| read::external(e, &mut errs, is_judge)),
            );
            nodes.resources.push(
                src.sub(key::RESOURCES, &mut errs)
                    .map(|r| read::resources(r, &mut errs)),
            );
            nodes.blackboard.push(
                src.sub(key::BLACKBOARD, &mut errs)
                    .and_then(|b| read::blackboard(b, &mut errs))
                    .map(|b| Blackboard {
                        read: b.read || is_judge,
                        ..b
                    }),
            );
            nodes
                .media
                .push(src.sub(key::MEDIA, &mut errs).map(|m| read::media(m, &mut errs)));
            nodes
                .message
                .push(src.sub(key::MESSAGE, &mut errs).map(|m| read::message(m, &mut errs)));
            nodes.validation.push(if is_judge {
                read::validation(src, &mut errs)
            } else {
                None
            });
        }
        errs.finish(nodes)
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// The entity of a node id.
    pub fn entity(&self, id: &str) -> Option<Entity> {
        self.by_id.get(id).copied()
    }

    /// The id of a node.
    pub fn id(&self, e: Entity) -> Option<&str> {
        self.ids.get(e.0 as usize).map(String::as_str)
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> {
        (0..self.ids.len() as u32).map(Entity)
    }

    pub fn model(&self) -> &Components<Model> {
        &self.model
    }

    pub fn external(&self) -> &Components<External> {
        &self.external
    }

    pub fn resources(&self) -> &Components<Resources> {
        &self.resources
    }

    pub fn blackboard(&self) -> &Components<Blackboard> {
        &self.blackboard
    }

    pub fn media(&self) -> &Components<Media> {
        &self.media
    }

    pub fn message(&self) -> &Components<Message> {
        &self.message
    }

    pub fn validation(&self) -> &Components<Validation> {
        &self.validation
    }
}
