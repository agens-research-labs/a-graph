//! The JSON Schemas (2020-12) generated from the shapes and their descriptions: of the graph,
//! `spec/graph.schema.json`, printed by `a-graph --print-schema`, and of the file of the
//! service, `spec/service.schema.json`, printed by `a-graph --print-service-schema`. They
//! say what the Loader checks, nothing more: the rules of the managers are in the descriptions.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{DOCUMENT, Field, SERVICE, Shape};

/// The schema of the graph, as the text of `spec/graph.schema.json`.
pub fn json_schema() -> String {
    schema_of(
        DOCUMENT,
        "a-graph graph file",
        "A graph of agents, run by a-graph. Generated from the code by `a-graph --print-schema`: do not edit it by hand.",
    )
}

/// The schema of the file of the service, as the text of `spec/service.schema.json`.
pub fn service_json_schema() -> String {
    schema_of(
        SERVICE,
        "a-graph service file",
        "The configuration of the service, `a-graph --serve service.yml`. Generated from the code by `a-graph --print-service-schema`: do not edit it by hand.",
    )
}

fn schema_of(fields: &[Field], title: &str, description: &str) -> String {
    let mut defs = BTreeMap::new();
    let mut root = map(fields, &mut defs);
    root.insert("$schema".into(), json!("https://json-schema.org/draft/2020-12/schema"));
    root.insert("title".into(), json!(title));
    root.insert("description".into(), json!(description));
    if !defs.is_empty() {
        root.insert("$defs".into(), Value::Object(defs.into_iter().collect()));
    }
    serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default() + "\n"
}

/// The definitions of the named shapes (`Shape::Lazy`), filled while they are met.
type Defs = BTreeMap<String, Value>;

fn shape(s: &Shape, defs: &mut Defs) -> Value {
    match s {
        Shape::Text | Shape::Id(_) | Shape::Ref(_) => json!({ "type": "string" }),
        Shape::Int => json!({ "type": "integer" }),
        Shape::Number => json!({ "type": "number" }),
        Shape::Bool => json!({ "type": "boolean" }),
        Shape::Words(words) => json!({ "enum": words_enum(words) }),
        Shape::List(inner) => json!({ "type": "array", "items": shape(inner, defs) }),
        Shape::Map(fields) => Value::Object(map(fields, defs)),
        Shape::MapOf(inner) => json!({ "type": "object", "additionalProperties": shape(inner, defs) }),
        Shape::Tagged { by, by_doc, common, variants } => {
            let names: Vec<&str> = variants.iter().map(|v| v.name).collect();
            let branches: Vec<Value> = variants
                .iter()
                .map(|v| {
                    let mut branch = map(&[common, v.fields].concat(), defs);
                    let props = branch.get_mut("properties").and_then(Value::as_object_mut).expect("a map has properties");
                    props.insert((*by).into(), json!({ "const": v.name }));
                    let required = branch.get_mut("required").and_then(Value::as_array_mut).expect("a map has required");
                    required.insert(0, json!(by));
                    branch.insert("description".into(), json!(v.doc));
                    Value::Object(branch)
                })
                .collect();
            json!({
                "type": "object",
                "required": [by],
                "properties": { *by: { "description": by_doc, "enum": names } },
                "oneOf": branches,
            })
        }
        Shape::Either(alternatives) => json!({ "anyOf": alternatives.iter().map(|a| shape(a, defs)).collect::<Vec<_>>() }),
        Shape::Lazy { name, doc, get } => {
            if !defs.contains_key(*name) {
                // reserved first: a shape that contains itself stops here
                defs.insert(name.to_string(), Value::Null);
                let mut def = shape(get(), defs);
                if let Value::Object(o) = &mut def {
                    o.insert("description".into(), json!(doc));
                }
                defs.insert(name.to_string(), def);
            }
            json!({ "$ref": format!("#/$defs/{name}") })
        }
        Shape::Any => json!({}),
    }
}

/// A map with these keys and no others.
fn map(fields: &[Field], defs: &mut Defs) -> Map<String, Value> {
    let mut properties = Map::new();
    for f in fields {
        let mut p = shape(&f.shape, defs);
        if let (false, Value::Object(o)) = (f.doc.is_empty(), &mut p) {
            o.insert("description".into(), json!(f.doc));
        }
        properties.insert(f.name.into(), p);
    }
    let required: Vec<&str> = fields.iter().filter(|f| f.required).map(|f| f.name).collect();
    let mut m = Map::new();
    m.insert("type".into(), json!("object"));
    m.insert("properties".into(), Value::Object(properties));
    m.insert("required".into(), json!(required));
    m.insert("additionalProperties".into(), json!(false));
    m
}

/// The Loader takes a number written where a word is expected as the word it spells: a word
/// that is a whole number (`"512"`) is accepted as a number too.
fn words_enum(words: &[&str]) -> Vec<Value> {
    let mut out: Vec<Value> = words.iter().map(|w| json!(w)).collect();
    out.extend(words.iter().filter_map(|w| w.parse::<i64>().ok()).map(|n| json!(n)));
    out
}
