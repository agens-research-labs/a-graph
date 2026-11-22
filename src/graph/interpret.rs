//! The interpretation of the nodes: the rules that need the managers. A node is checked against
//! the model it uses (its parameters, what the model can do), the prompt it binds (placeholders)
//! and the media it produces.

use std::collections::{HashMap, HashSet};

use super::edges::{Edges, Group, Pattern, Step};
use super::nodes::component::{InFrom, MessageInput, Model as NodeModel, OutFrom, Validation};
use super::nodes::{Entity, Nodes};
use crate::error::{Error, Path, Phase, Rule};
use crate::external::External;
use crate::models::{Llm, ModelManager, ModelRequirements, ProducedMedia};
use crate::resources::{MediaKind, Origin, PromptBody, Resources};
use crate::schema::key;

/// Every rule of the nodes that needs a manager. All the errors are returned, not only the first.
pub fn interpret(
    nodes: &Nodes,
    edges: &Edges,
    models: &ModelManager,
    resources: &Resources,
    external: &External,
) -> Vec<Error> {
    let mut errs = Vec::new();
    let in_batch = batch_nodes(nodes, edges, &mut errs);
    let previous = previous_nodes(edges);
    let controllers = controllers(nodes, edges);
    // media id -> the node that writes it: one media has one writer
    let mut writers: HashMap<String, String> = HashMap::new();
    for e in nodes.entities() {
        let at = Path::root().key(key::GRAPH).key(key::NODES).idx(e.0 as usize);
        let id = nodes.id(e).unwrap_or_default();
        // a validation node is read like a node, from inside `validation-node`
        let base = if nodes.validation().get(e).is_some() {
            at.key(key::VALIDATION_NODE)
        } else {
            at.clone()
        };
        let Some(m) = nodes.model().get(e) else {
            errs.push(err(
                base.key(key::MODEL),
                Rule::MissingField,
                format!("node `{id}` needs `model` (or `validation-node`)"),
            ));
            continue;
        };
        let asked = Asked {
            // a react controller answers with the routing output, which the model has to be able to give
            schema: controllers
                .get(&e)
                .cloned()
                .or_else(|| nodes.validation().get(e).map(|_| Validation::judge_schema()))
                .or_else(|| {
                    nodes
                        .message()
                        .get(e)
                        .and_then(|x| x.output.as_ref())
                        .and_then(|o| o.schema.clone())
                }),
            logprobs: nodes
                .message()
                .get(e)
                .and_then(|x| x.output.as_ref())
                .and_then(|o| o.logprobs),
            input_media: input_kinds(nodes, e, resources),
            output_media: produced(nodes, e, id, &at, resources, &mut writers, &mut errs),
            uses_tools: uses_tools(nodes, e),
            batch: in_batch.contains(&e),
        };
        check_model(&base.key(key::MODEL), m, asked, models, &mut errs);
        skills_rules(nodes, e, id, &base, resources, &mut errs);
        placeholders(nodes, e, id, &base, resources, external, &mut errs);
        board_rules(nodes, e, id, &base, &mut errs);
        media_input_rules(nodes, e, &at, resources, &previous, &mut errs);
    }
    errs
}

fn err(path: Path, rule: Rule, msg: impl Into<String>) -> Error {
    Error::new(Phase::Interpretation, path, rule, msg)
}

/// Tools, MCP servers or skills are bound to the node.
fn uses_tools(nodes: &Nodes, e: Entity) -> bool {
    nodes
        .resources()
        .get(e)
        .is_some_and(|r| !r.tools.is_empty() || !r.mcp_servers.is_empty() || !r.skills.is_empty())
}

/// What the node asks of its model besides its own parameters.
#[derive(Default)]
struct Asked {
    schema: Option<serde_json::Value>,
    logprobs: Option<u32>,
    input_media: Vec<MediaKind>,
    output_media: Vec<ProducedMedia>,
    uses_tools: bool,
    /// The node runs under a `batch` group.
    batch: bool,
}

/// The types of the media with these ids.
/// The types of the media a node reads, as the model gets them: what a utility changes is text.
fn input_kinds(nodes: &Nodes, e: Entity, resources: &Resources) -> Vec<MediaKind> {
    let Some(m) = nodes.media().get(e) else {
        return Vec::new();
    };
    m.input
        .iter()
        .filter_map(|i| {
            resources.media.get(&i.id).map(|r| {
                if i.utility.is_empty() {
                    r.kind.clone()
                } else {
                    MediaKind::Md
                }
            })
        })
        .collect()
}

/// What the model can do with what the node asks (`Llm::check`, no network): the model knows its
/// own limits, such as a mandatory `max_tokens` or the family of the media provider fields.
fn check_model(at: &Path, m: &NodeModel, asked: Asked, models: &ModelManager, errs: &mut Vec<Error>) {
    let Some(llm) = models.get(&m.id) else { return };
    let req = ModelRequirements {
        temperature: m.temperature,
        top_p: m.top_p,
        max_tokens: m.max_tokens,
        reasoning: m.reasoning,
        logprobs: asked.logprobs,
        schema: asked.schema,
        input_media: asked.input_media,
        output_media: asked.output_media,
        uses_tools: asked.uses_tools,
        batch: asked.batch,
    };
    if let Err(e) = Llm::check(llm, &req) {
        errs.push(err(at.clone(), Rule::NotSupported, e.to_string()));
    }
}

/// The model calls the skills through a tool of the program, `activate_skill`, and tells them by
/// the `name` of their SKILL.md: the node cannot have a tool with that name, nor two skills with the
/// same name.
fn skills_rules(nodes: &Nodes, e: Entity, id: &str, at: &Path, resources: &Resources, errs: &mut Vec<Error>) {
    let Some(r) = nodes.resources().get(e) else { return };
    if r.skills.is_empty() {
        return;
    }
    let here = at.key(key::RESOURCES);
    for reserved in [super::ACTIVATE_SKILL, super::READ_SKILL_FILE, super::RUN_SKILL_SCRIPT] {
        if r.tools.iter().any(|t| t == reserved) {
            errs.push(err(here.key(key::TOOLS), Rule::DuplicateId, format!("node `{id}` has skills, so `{reserved}` is a tool of the program: a tool of the node cannot have that name")));
        }
    }
    let mut names: HashSet<&str> = HashSet::new();
    for (i, sid) in r.skills.iter().enumerate() {
        let Some(skill) = resources.skills.get(sid) else {
            continue;
        };
        if !names.insert(skill.name.as_str()) {
            errs.push(err(
                here.key(key::SKILLS).idx(i),
                Rule::DuplicateId,
                format!(
                    "node `{id}` has two skills called `{}`: the model could not tell them apart",
                    skill.name
                ),
            ));
        }
    }
}

/// The media a node produces. One made by the model (`from: model`) must have `generate`; one made by
/// a utility from the text (`from: message`) must be of type `file` and have a utility. Each is
/// written by one node only. Whether the model can produce its own is for `Llm::check`.
fn produced(
    nodes: &Nodes,
    e: Entity,
    id: &str,
    at: &Path,
    resources: &Resources,
    writers: &mut HashMap<String, String>,
    errs: &mut Vec<Error>,
) -> Vec<ProducedMedia> {
    let mut out = Vec::new();
    let Some(media) = nodes.media().get(e) else { return out };
    for (k, entry) in media.output.iter().enumerate() {
        let ip = at.key(key::MEDIA).key(key::OUTPUT).idx(k);
        let Some(resource) = resources.media.get(&entry.id) else {
            continue;
        };
        match (entry.from, &resource.origin) {
            (OutFrom::Model, Origin::Generated { generate, provider }) => {
                out.push(ProducedMedia {
                    kind: resource.kind.clone(),
                    generate: generate.clone(),
                    provider: provider.clone(),
                });
            }
            (OutFrom::Model, _) => {
                errs.push(err(
                    ip.clone(),
                    Rule::MediaNotGenerated,
                    format!("media `{}` has no `generate`: the model cannot produce it", entry.id),
                ));
                continue;
            }
            (OutFrom::Message, Origin::MadeByUtility) => {
                if entry.utility.is_empty() {
                    errs.push(err(
                        ip.clone(),
                        Rule::MediaNeedsUtility,
                        format!("media `{}` is made from the text: it needs a `utility`", entry.id),
                    ));
                }
            }
            (OutFrom::Message, _) => {
                errs.push(err(
                    ip.clone(),
                    Rule::MediaNotMade,
                    format!(
                        "media `{}` is not of type `file`: only a media of that type can be made from the text",
                        entry.id
                    ),
                ));
                continue;
            }
        }
        if let Some(first) = writers.insert(entry.id.clone(), id.to_string()) {
            errs.push(err(
                ip,
                Rule::MediaTwoWriters,
                format!("media `{}` is already written by node `{first}`", entry.id),
            ));
        }
    }
    out
}

/// The media a node reads: one from `resources` must have a source; one from `node` must be produced by a node
/// connected before.
fn media_input_rules(
    nodes: &Nodes,
    e: Entity,
    at: &Path,
    resources: &Resources,
    previous: &HashMap<Entity, Vec<Entity>>,
    errs: &mut Vec<Error>,
) {
    let Some(media) = nodes.media().get(e) else { return };
    for (k, entry) in media.input.iter().enumerate() {
        let p = at.key(key::MEDIA).key(key::INPUT).idx(k);
        let Some(resource) = resources.media.get(&entry.id) else {
            continue;
        };
        match entry.from {
            InFrom::Resources => match &resource.origin {
                Origin::Source(_) => {}
                Origin::Generated { .. } | Origin::MadeByUtility => {
                    errs.push(err(
                        p.clone(),
                        Rule::MediaInputGenerated,
                        format!(
                            "media `{}` is made by a node: it is `from: node`, not from the resources",
                            entry.id
                        ),
                    ));
                }
            },
            InFrom::Node => {
                let produced_before = previous.get(&e).is_some_and(|nodes_before| {
                    nodes_before.iter().any(|b| {
                        nodes
                            .media()
                            .get(*b)
                            .is_some_and(|m| m.output.iter().any(|o| o.id == entry.id))
                    })
                });
                if !produced_before {
                    errs.push(err(
                        p.clone(),
                        Rule::MediaNotFromNode,
                        format!("no node connected before produces media `{}`", entry.id),
                    ));
                }
            }
        }
    }
}

/// For every node, the nodes whose output can reach it: the one before it in the sequence, the `in` node of
/// a group for each of its branches, the branches for the `out` node.
fn previous_nodes(edges: &Edges) -> HashMap<Entity, Vec<Entity>> {
    fn walk(step: &Step, from: Vec<Entity>, map: &mut HashMap<Entity, Vec<Entity>>) -> Vec<Entity> {
        match step {
            Step::Node(x) => {
                map.entry(*x).or_default().extend(from);
                vec![*x]
            }
            Step::Group(g) => {
                map.entry(g.r#in).or_default().extend(from);
                let mut exits = Vec::new();
                for s in &g.fan {
                    exits.extend(walk(s, vec![g.r#in], map));
                }
                match g.out {
                    Some(out) => {
                        map.entry(out).or_default().extend(exits);
                        vec![out]
                    }
                    None => Vec::new(),
                }
            }
            Step::React(r) => {
                map.entry(r.controller).or_default().extend(from);
                vec![r.controller]
            }
        }
    }
    let mut map = HashMap::new();
    let mut carry = vec![edges.start()];
    for step in edges.steps() {
        carry = walk(step, carry, &mut map);
    }
    if let Some(end) = edges.end() {
        map.entry(end).or_default().extend(carry);
    }
    map
}

/// A node that writes to its board and cleans it in the same task would lose what it wrote.
fn board_rules(nodes: &Nodes, e: Entity, id: &str, at: &Path, errs: &mut Vec<Error>) {
    if let Some(b) = nodes
        .blackboard()
        .get(e)
        .filter(|b| b.cleanup && b.write.as_ref().is_some_and(|w| w.enabled))
    {
        errs.push(err(
            at.key(key::BLACKBOARD).key(key::CLEANUP),
            Rule::CleanupWrite,
            format!(
                "node `{id}` writes board `{}` and cleans it in the same task: what it writes would be lost",
                b.id
            ),
        ));
    }
}

/// The placeholder that takes the chunks found in the vector databases of the node.
pub const CHUNKS: &str = "chunks";

/// The placeholders that take the messages of the node.
const MESSAGES: [&str; 3] = [key::USER_MESSAGE, key::AGENT_MESSAGE, "input_message"];

/// `{{name}}` placeholders of the prompt against what the node declares.
fn placeholders(
    nodes: &Nodes,
    e: Entity,
    id: &str,
    at: &Path,
    resources: &Resources,
    external: &External,
    errs: &mut Vec<Error>,
) {
    // declared names: the ids of what the node reads from outside, in order of appearance
    let mut declared: Vec<&str> = Vec::new();
    if let Some(ext) = nodes.external().get(e) {
        declared.extend(ext.search.iter().map(|q| q.id.as_str()));
        // only a memory that makes summaries has something to put in the prompt: the summary
        declared.extend(
            ext.memory
                .iter()
                .filter(|m| external.memory.get(&m.id).is_some_and(|c| c.synt.is_some()))
                .map(|m| m.id.as_str()),
        );
    }
    // a board the node does not read gives nothing to the prompt: it has no placeholder
    if let Some(b) = nodes.blackboard().get(e).filter(|b| b.read) {
        declared.push(&b.id);
    }
    // the chunks found in the vector databases come in one placeholder with a fixed name
    let reads_vectordb = nodes.external().get(e).is_some_and(|ext| !ext.vectordb.is_empty());
    if let Some(ext) = nodes.external().get(e) {
        let mut seen_db = HashSet::new();
        for q in &ext.vectordb {
            if !seen_db.insert(q.id.as_str()) {
                errs.push(err(
                    at.key(key::EXTERNAL),
                    Rule::DuplicateId,
                    format!("vectordb `{}` is declared twice on node `{id}`", q.id),
                ));
            }
        }
    }
    if declared.contains(&CHUNKS) {
        errs.push(err(at.key(key::EXTERNAL), Rule::DuplicateId, format!("`{CHUNKS}` is the placeholder of the vectordb chunks: it cannot be the id of something else on node `{id}`")));
    }
    for name in MESSAGES {
        if declared.contains(&name) {
            errs.push(err(at.key(key::EXTERNAL), Rule::DuplicateId, format!("`{name}` is the placeholder of the message of the node: it cannot be the id of something else on node `{id}`")));
        }
    }
    let mut seen = HashSet::new();
    for d in &declared {
        if !seen.insert(*d) {
            errs.push(err(
                at.key(key::EXTERNAL),
                Rule::DuplicateId,
                format!("`{d}` is declared twice on node `{id}`: placeholders would be ambiguous"),
            ));
        }
    }

    let Some(prompt_id) = nodes.resources().get(e).and_then(|r| r.prompt.as_deref()) else {
        if reads_vectordb {
            errs.push(err(
                at.key(key::EXTERNAL),
                Rule::PlaceholderUnused,
                format!("node `{id}` reads a vectordb and has no prompt that uses {{{{{CHUNKS}}}}}"),
            ));
        }
        let m = nodes.message().get(e).and_then(|m| m.input).unwrap_or_default();
        if m.user_message && m.agent_message {
            errs.push(err(
                at.key(key::MESSAGE),
                Rule::PromptRequired,
                format!("node `{id}` takes both messages and has no prompt: the model could not tell them apart"),
            ));
        }
        return;
    };
    let Some(prompt) = resources.prompts.get(prompt_id) else {
        return;
    };
    // a prompt not read yet (url) is checked when it is
    let PromptBody::Content(content) = &prompt.body else {
        return;
    };
    let input = nodes.message().get(e).and_then(|m| m.input).unwrap_or_default();
    let pp = at.key(key::RESOURCES).key(key::PROMPT);
    check_placeholders(
        &pp,
        prompt_id,
        [&content.system, &content.user],
        &declared,
        reads_vectordb,
        input,
        errs,
    );
}

fn check_placeholders(
    p: &Path,
    prompt: &str,
    texts: [&String; 2],
    declared: &[&str],
    reads_vectordb: bool,
    input: MessageInput,
    errs: &mut Vec<Error>,
) {
    let mut used: Vec<String> = Vec::new();
    for text in texts {
        match scan(text) {
            Ok(names) => used.extend(names),
            Err(msg) => errs.push(err(
                p.clone(),
                Rule::PlaceholderMalformed,
                format!("prompt `{prompt}`: {msg}"),
            )),
        }
    }
    let has = |n: &str| used.iter().any(|u| u == n);
    let (u, a) = (input.user_message, input.agent_message);

    for name in &used {
        let ok = declared.contains(&name.as_str())
            || match name.as_str() {
                "user_message" => u,
                "agent_message" => a,
                "input_message" => u != a,
                CHUNKS => reads_vectordb,
                _ => false,
            };
        if !ok {
            errs.push(err(
                p.clone(),
                Rule::PlaceholderUnbound,
                format!("prompt `{prompt}` uses {{{{{name}}}}}, which this node does not declare"),
            ));
        }
    }
    for d in declared {
        if !has(d) {
            errs.push(err(
                p.clone(),
                Rule::PlaceholderUnused,
                format!("prompt `{prompt}` never uses {{{{{d}}}}}, declared on the node"),
            ));
        }
    }
    if reads_vectordb && !has(CHUNKS) {
        errs.push(err(
            p.clone(),
            Rule::PlaceholderUnused,
            format!("prompt `{prompt}` never uses {{{{{CHUNKS}}}}}, the node reads a vectordb"),
        ));
    }
    if has("input_message") && (has("user_message") || has("agent_message")) {
        errs.push(err(
            p.clone(),
            Rule::PlaceholderInputMixed,
            "{{input_message}} cannot be used together with {{user_message}} or {{agent_message}}",
        ));
    }
    if u && a {
        for n in ["user_message", "agent_message"] {
            if !has(n) {
                errs.push(err(
                    p.clone(),
                    Rule::PlaceholderUnused,
                    format!("prompt `{prompt}` never uses {{{{{n}}}}}, enabled on the node"),
                ));
            }
        }
    } else if (u || a) && !(has("input_message") || has("user_message") || has("agent_message")) {
        errs.push(err(
            p.clone(),
            Rule::PlaceholderUnused,
            format!("prompt `{prompt}` does not use the node's message"),
        ));
    }
}

/// Names found in `{{name}}`. Err on an unclosed `{{` or an invalid name.
pub(crate) fn scan(text: &str) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            return Err("unclosed `{{`".to_string());
        };
        let name = after[..end].trim();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(format!("invalid placeholder name `{name}`"));
        }
        names.push(name.to_string());
        rest = &after[end + 2..];
    }
    Ok(names)
}

/// The nodes that run under a `batch` group (in its `fan`, nested groups included). They use batch
/// inference, which only makes sense with one model: they must all reference the same one.
fn batch_nodes(nodes: &Nodes, edges: &Edges, errs: &mut Vec<Error>) -> HashSet<Entity> {
    let mut all = HashSet::new();
    let mut groups = Vec::new();
    collect(edges.steps(), &mut groups);
    for g in groups.into_iter().filter(|g| g.pattern == Pattern::Batch) {
        let mut members = Vec::new();
        for step in &g.fan {
            // a batch is one request per node: a branch with a group or a react inside cannot be one
            if !matches!(step, Step::Node(_)) {
                let at = Path::root().key(key::GRAPH).key(key::EDGES);
                errs.push(err(at, Rule::BatchNested, format!("the batch group of `{}` has a group or a react among its branches: the branches of a batch are plain nodes", nodes.id(g.r#in).unwrap_or_default())));
            }
            inside(step, &mut members);
        }
        // the model of each member, by id; the first one is the model of the group
        let models: Vec<(Entity, &str)> = members
            .iter()
            .filter_map(|e| nodes.model().get(*e).map(|m| (*e, m.id.as_str())))
            .collect();
        if let Some(&(first, wanted)) = models.first() {
            for &(e, model) in &models[1..] {
                if model != wanted {
                    let at = Path::root()
                        .key(key::GRAPH)
                        .key(key::NODES)
                        .idx(e.0 as usize)
                        .key(key::MODEL)
                        .key(key::ID);
                    errs.push(err(
                        at,
                        Rule::BatchModels,
                        format!("node `{}` uses model `{model}`, but the nodes of its batch must use the same model as `{}`: `{wanted}`", nodes.id(e).unwrap_or_default(), nodes.id(first).unwrap_or_default()),
                    ));
                }
            }
        }
        // a batch answer is the last one: a tool call in it would never run
        for &e in &members {
            if uses_tools(nodes, e) {
                let at = Path::root()
                    .key(key::GRAPH)
                    .key(key::NODES)
                    .idx(e.0 as usize)
                    .key(key::RESOURCES);
                errs.push(err(
                    at,
                    Rule::BatchTools,
                    format!("node `{}` is in a batch and has tools, MCP servers or skills: the answer of a batch is the last one, so its tool calls would never run", nodes.id(e).unwrap_or_default()),
                ));
            }
            // a batch answer is the last one too: a correction of `schema_prompt` could never be sent
            let prompt = nodes
                .message()
                .get(e)
                .and_then(|m| m.output.as_ref())
                .is_some_and(|o| o.schema_prompt.is_some());
            if prompt {
                let at = Path::root()
                    .key(key::GRAPH)
                    .key(key::NODES)
                    .idx(e.0 as usize)
                    .key(key::MESSAGE)
                    .key(key::OUTPUT)
                    .key(key::SCHEMA_PROMPT);
                errs.push(err(
                    at,
                    Rule::BatchSchemaPrompt,
                    format!("node `{}` is in a batch and has a `schema_prompt`: the answer of a batch is the last one, so a correction could never be sent; use `schema`, or take the node out of the batch", nodes.id(e).unwrap_or_default()),
                ));
            }
        }
        all.extend(members);
    }
    all
}

/// Every group of the list, the nested ones too.
fn collect<'a>(steps: &'a [Step], out: &mut Vec<&'a Group>) {
    for step in steps {
        match step {
            Step::Node(_) => {}
            Step::Group(g) => {
                out.push(g);
                collect(&g.fan, out);
            }
            Step::React(r) => collect(&r.workers, out),
        }
    }
}

/// Every node of a step, nested ones included.
fn inside(step: &Step, out: &mut Vec<Entity>) {
    match step {
        Step::Node(x) => out.push(*x),
        Step::Group(g) => {
            out.push(g.r#in);
            g.fan.iter().for_each(|s| inside(s, out));
            out.extend(g.out);
        }
        Step::React(r) => {
            out.push(r.controller);
            r.workers.iter().for_each(|s| inside(s, out));
        }
    }
}

/// The react controllers and the routing schema each one answers with.
fn controllers(nodes: &Nodes, edges: &Edges) -> HashMap<Entity, serde_json::Value> {
    fn walk(steps: &[Step], nodes: &Nodes, out: &mut HashMap<Entity, serde_json::Value>) {
        for step in steps {
            match step {
                Step::Node(_) => {}
                Step::Group(g) => walk(&g.fan, nodes, out),
                Step::React(r) => {
                    out.insert(r.controller, super::edges::React::routing_schema(&r.worker_ids(nodes)));
                    walk(&r.workers, nodes, out);
                }
            }
        }
    }
    let mut out = HashMap::new();
    walk(edges.steps(), nodes, &mut out);
    out
}
