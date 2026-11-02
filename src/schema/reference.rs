//! The references in Markdown, generated from the shapes and their descriptions: of the graph,
//! `docs/graph.md`, printed by `a-graph --print-docs`, and of the file of the service,
//! `docs/service.md`, printed by `a-graph --print-service-docs`. Every map has its own
//! section, named by its path; a named shape (`Shape::Lazy`) has one section, at the end, named by
//! its name.

use std::fmt::Write;

use super::key::ENV;
use super::{DOCUMENT, Field, SERVICE, Shape};

/// The reference of the graph, as the text of `docs/graph.md`.
pub fn reference() -> String {
    page(HEADER, DOCUMENT)
}

/// The reference of the file of the service, as the text of `docs/service.md`.
pub fn service_reference() -> String {
    page(SERVICE_HEADER, SERVICE)
}

fn page(header: &str, fields: &[Field]) -> String {
    let mut r = Reference::default();
    r.out.push_str(header);
    r.section("The file", fields, "");
    r.fields_below("", fields);
    let mut done = Vec::new();
    while let Some((name, doc, get)) = r.named.iter().find(|(n, ..)| !done.contains(n)).copied() {
        done.push(name);
        let _ = write!(r.out, "\n## `{name}`\n\n{doc}\n");
        r.below(name, get(), true);
    }
    r.out
}

const HEADER: &str = "# The reference

Every key of the graph file. This page is generated from the code by `a-graph --print-docs`: do not edit it by hand.
The same keys are in the JSON Schema `spec/graph.schema.json`, printed by `a-graph --print-schema`.

How the file works as a whole is explained in [the graph file](file.md). In the paths, `[]` is an item of a list and
`<name>` a key you choose. A secret is written `{ env: <VARIABLE> }`: the name of the environment variable that holds
its value.
";

const SERVICE_HEADER: &str = "# The service file

The keys of the file that starts the service, `a-graph --serve service.yml`. This page is generated from the code by
`a-graph --print-service-docs`: do not edit it by hand. The same keys are in the JSON Schema
`spec/service.schema.json`, printed by `a-graph --print-service-schema`.

The file holds no keys of models or services: those are variables of the environment of the service, which a graph
names as always, `api_key: { env: ANTHROPIC_API_KEY }`. The HTTP API of the service is described, in OpenAPI 3.1, in
`spec/service.openapi.json`, printed by `a-graph --print-service-api`.
";

/// A named shape: its name, its description and how to reach it.
type Named = (&'static str, &'static str, fn() -> &'static Shape);

#[derive(Default)]
struct Reference {
    out: String,
    /// The named shapes met so far, in order.
    named: Vec<Named>,
    /// Added to the titles of the sections below a variant: `with `provider: bedrock``.
    context: String,
    /// The forms being described are the entries of a list.
    entries: bool,
}

impl Reference {
    /// The table of the keys of a map.
    fn section(&mut self, title: &str, fields: &[Field], intro: &str) {
        self.heading(title);
        if !intro.is_empty() {
            let _ = write!(self.out, "{intro}\n\n");
        }
        self.table(fields);
    }

    fn heading(&mut self, title: &str) {
        let _ = write!(self.out, "\n## {title}\n\n");
    }

    /// The title of the section of a map at `path`.
    fn title(&self, path: &str) -> String {
        format!("`{path}`{}", self.context)
    }

    fn table(&mut self, fields: &[Field]) {
        self.table_with(fields, None);
    }

    /// The table of the keys of which a map has exactly one: each is mandatory in its own form.
    fn keys_table(&mut self, fields: &[Field]) {
        self.out.push_str("| Key | Value | Description |\n|---|---|---|\n");
        for f in fields {
            let value = self.value(&f.shape);
            let _ = writeln!(self.out, "| `{}` | {} | {} |", f.name, cell(&value), cell(f.doc));
        }
    }

    /// The table, with `first` as its first row when given.
    fn table_with(&mut self, fields: &[Field], first: Option<String>) {
        self.out
            .push_str("| Key | Required | Value | Description |\n|---|---|---|---|\n");
        if let Some(row) = first {
            let _ = writeln!(self.out, "{row}");
        }
        for f in fields {
            let required = if f.required { "yes" } else { "no" };
            let value = self.value(&f.shape);
            let _ = writeln!(
                self.out,
                "| `{}` | {required} | {} | {} |",
                f.name,
                cell(&value),
                cell(f.doc)
            );
        }
    }

    /// The sections of the maps inside the fields of a map at `path`.
    fn fields_below(&mut self, path: &str, fields: &[Field]) {
        for f in fields {
            let p = if path.is_empty() {
                f.name.to_string()
            } else {
                format!("{path}.{}", f.name)
            };
            self.below(&p, &f.shape, false);
        }
    }

    /// The sections of the maps a value at `path` holds. `named`: the section of a named shape,
    /// whose own heading is already written.
    fn below(&mut self, path: &str, shape: &Shape, named: bool) {
        match shape {
            Shape::List(inner) if matches!(inner, Shape::Either(_)) => {
                let entries = std::mem::replace(&mut self.entries, true);
                self.below(&format!("{path}[]"), inner, named);
                self.entries = entries;
            }
            Shape::List(inner) => self.below(&format!("{path}[]"), inner, named),
            Shape::MapOf(inner) => self.below(&format!("{path}.<name>"), inner, named),
            Shape::Map(fields) if is_secret(fields) => {}
            Shape::Map(fields) => {
                if named {
                    self.out.push('\n');
                    self.table(fields);
                } else {
                    self.section(&self.title(path), fields, "");
                }
                self.fields_below(path, fields);
            }
            Shape::Tagged {
                by,
                by_doc,
                common,
                variants,
            } => {
                let names: Vec<String> = variants.iter().map(|v| format!("`{}`", v.name)).collect();
                self.heading(&self.title(path));
                let _ = write!(
                    self.out,
                    "The key `{by}` chooses the other keys, described in a section for each value.\n\n"
                );
                let row = format!("| `{by}` | yes | {} | {} |", or_list(&names), cell(by_doc));
                self.table_with(common, Some(row));
                self.fields_below(path, common);
                for v in *variants {
                    let outer = std::mem::replace(&mut self.context, format!(" with `{by}: {}`", v.name));
                    self.section(&self.title(path), v.fields, v.doc);
                    self.fields_below(path, v.fields);
                    self.context = outer;
                }
            }
            Shape::Either(alternatives) => {
                let maps: Vec<&[Field]> = alternatives
                    .iter()
                    .filter_map(|a| {
                        if let Shape::Map(f) = a {
                            (!is_secret(f)).then_some(*f)
                        } else {
                            None
                        }
                    })
                    .collect();
                if maps.is_empty() {
                    return;
                }
                if !named {
                    self.heading(&self.title(path));
                } else {
                    self.out.push('\n');
                }
                if maps.len() == 1 {
                    self.table(maps[0]);
                } else if maps.iter().all(|f| f.len() == 1) {
                    let what = if self.entries { "Each entry is a map" } else { "A map" };
                    let _ = write!(self.out, "{what} with exactly one of these keys.\n\n");
                    let all: Vec<Field> = maps.iter().map(|f| f[0]).collect();
                    self.keys_table(&all);
                } else {
                    for (i, fields) in maps.iter().enumerate() {
                        if i > 0 {
                            self.out.push('\n');
                        }
                        let keys: Vec<String> = fields.iter().map(|f| format!("`{}`", f.name)).collect();
                        let _ = write!(self.out, "The form with {}:\n\n", and_list(&keys));
                        self.table(fields);
                    }
                }
                for fields in maps {
                    self.fields_below(path, fields);
                }
            }
            _ => {}
        }
    }

    /// What the column `Value` says of a shape.
    fn value(&mut self, shape: &Shape) -> String {
        match shape {
            Shape::Text => "text".into(),
            Shape::Int => "whole number".into(),
            Shape::Number => "number".into(),
            Shape::Bool => "`true` or `false`".into(),
            Shape::Words(words) => or_list(&words.iter().map(|w| format!("`{w}`")).collect::<Vec<_>>()),
            Shape::Id(kind) => format!("the id of this {}", kind_name(*kind)),
            Shape::Ref(kind) => {
                let name = kind_name(*kind);
                let article = if name.starts_with(['a', 'e', 'i', 'o', 'u', 'M']) {
                    "an"
                } else {
                    "a"
                };
                format!("the id of {article} {name}")
            }
            Shape::List(inner) => format!("list of: {}", self.value(inner)),
            Shape::Map(fields) if is_secret(fields) => "secret".into(),
            Shape::Map(_) | Shape::Tagged { .. } => "map (below)".into(),
            Shape::MapOf(inner) => format!("map of names to: {}", self.value(inner)),
            Shape::Either(alternatives) => {
                let mut parts: Vec<String> = Vec::new();
                for a in *alternatives {
                    let part = self.value(a);
                    if !parts.contains(&part) {
                        parts.push(part);
                    }
                }
                parts.join(", or ")
            }
            Shape::Lazy { name, doc, get } => {
                if !self.named.iter().any(|(n, ..)| n == name) {
                    self.named.push((name, doc, *get));
                }
                format!("a `{name}` (below)")
            }
            Shape::Any => "any value".into(),
        }
    }
}

/// The name of a kind of id, as the reference writes it.
fn kind_name(kind: super::Kind) -> &'static str {
    match kind {
        super::Kind::Mcp => "MCP server",
        other => other.name(),
    }
}

/// `{ env }`: a secret, described once in the header.
fn is_secret(fields: &[Field]) -> bool {
    fields.len() == 1 && fields[0].name == ENV
}

/// A text for one cell of a table.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

fn or_list(items: &[String]) -> String {
    join_last(items, " or ")
}

fn and_list(items: &[String]) -> String {
    join_last(items, " and ")
}

fn join_last(items: &[String], last: &str) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., tail] => format!("{}{last}{tail}", init.join(", ")),
    }
}
