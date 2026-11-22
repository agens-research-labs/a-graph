//! The tool loop of a node: the model is called, and while it asks for tools the gate calls them
//! and the results go back to the model, until it answers without asking. The graph sees only the
//! final response.

use std::collections::HashMap;

use futures_util::future::join_all;

use super::Program;
use super::node::logprobs_missing;
use super::structured::{Mechanism, Next, SchemaOf};
use crate::error::{Error, Path, Phase, Rule};
use crate::gate::{self, ToolOutcome};
use crate::graph::nodes::component::DEFAULT_MAX_ROUNDS;
use crate::graph::{ACTIVATE_SKILL, Entity, READ_SKILL_FILE, RUN_SKILL_SCRIPT};
use crate::logging::{Event, LogLevel};
use crate::models::{Llm, ModelInvocation, ModelResponse, ToolDefinition, ToolResult, ToolRound};

/// Where the call of a tool goes: one of the node's own tools, or a tool of one of its MCP servers.
enum Target<'a> {
    Tool(&'a crate::resources::Tool),
    /// The tool of the program that gives the instructions of a skill.
    Skill,
    /// The tool of the program that reads a file of the folder of a skill.
    SkillFile,
    /// The tool of the program that runs a script of a skill.
    SkillScript,
    Mcp {
        server: &'a crate::resources::McpServer,
        tool: String,
    },
}

impl Program {
    /// The tools of a node, as the model is told about them: its own, then those of its MCP servers.
    pub(super) fn tool_definitions(&self, node: Entity) -> Vec<ToolDefinition> {
        let Some(r) = self.nodes.resources().get(node) else {
            return Vec::new();
        };
        let mut all: Vec<ToolDefinition> = r
            .tools
            .iter()
            .filter_map(|id| self.resources.tools.get(id))
            .map(|t| ToolDefinition {
                name: t.id.clone(),
                description: t.description.clone(),
                parameters: serde_json::to_value(&t.parameters).unwrap_or_default(),
            })
            .collect();
        if let Some(pool) = self.mcp_pool() {
            for server in &r.mcp_servers {
                all.extend(pool.tools(server).iter().cloned());
            }
        }
        // the model may only name the skills the node has
        let names: Vec<String> = self.node_skills(node).iter().map(|s| s.name.clone()).collect();
        if !names.is_empty() {
            all.push(ToolDefinition {
                name: ACTIVATE_SKILL.to_string(),
                description: "Loads the full instructions of a skill".to_string(),
                parameters: serde_json::json!({ "type": "object", "properties": { "name": { "type": "string", "enum": names } }, "required": ["name"] }),
            });
            let runners: Vec<String> = self
                .node_skills(node)
                .iter()
                .filter(|s| s.run_scripts)
                .map(|s| s.name.clone())
                .collect();
            if !runners.is_empty() {
                all.push(ToolDefinition {
                    name: RUN_SKILL_SCRIPT.to_string(),
                    description: "Runs a script of the folder scripts/ of a skill, with a list of arguments".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": { "skill": { "type": "string", "enum": runners }, "path": { "type": "string" }, "args": { "type": "array", "items": { "type": "string" } } },
                        "required": ["skill", "path"]
                    }),
                });
            }
            all.push(ToolDefinition {
                name: READ_SKILL_FILE.to_string(),
                description: "Reads a file of the folder of a skill, given its path relative to that folder"
                    .to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": { "skill": { "type": "string", "enum": names }, "path": { "type": "string" } },
                    "required": ["skill", "path"]
                }),
            });
        }
        all
    }

    /// The skills of a node, by the name the model knows them: the one in their `SKILL.md`.
    pub(super) fn node_skills(&self, node: Entity) -> Vec<&crate::resources::Skill> {
        let Some(r) = self.nodes.resources().get(node) else {
            return Vec::new();
        };
        r.skills.iter().filter_map(|id| self.resources.skills.get(id)).collect()
    }

    /// What the system text of a node gets when the node has skills: how to use them and, for each,
    /// its name and description. The instructions of a skill come only when the model activates it.
    pub(super) fn skills_catalog(&self, node: Entity) -> Option<String> {
        let skills = self.node_skills(node);
        if skills.is_empty() {
            return None;
        }
        let mut text = format!(
            "The following skills provide specialized instructions for specific tasks.\nWhen a task matches the description of a skill, call the {ACTIVATE_SKILL} tool with the name of the skill to load its full instructions.\nWhen the instructions refer to other files of the skill, read them with the {READ_SKILL_FILE} tool.\n"
        );
        if skills.iter().any(|s| s.run_scripts) {
            text.push_str(&format!("When the instructions tell you to run a script of the skill, run it with the {RUN_SKILL_SCRIPT} tool.\n"));
        }
        text.push_str("\n<available_skills>\n");
        for s in skills {
            text.push_str(&format!(
                "  <skill>\n    <name>{}</name>\n    <description>{}</description>\n  </skill>\n",
                s.name, s.description
            ));
        }
        text.push_str("</available_skills>");
        Some(text)
    }

    /// Who answers a call of the model, if the node has such a tool.
    fn target(&self, node: Entity, name: &str) -> Option<Target<'_>> {
        let r = self.nodes.resources().get(node)?;
        if name == ACTIVATE_SKILL && !r.skills.is_empty() {
            return Some(Target::Skill);
        }
        if name == READ_SKILL_FILE && !r.skills.is_empty() {
            return Some(Target::SkillFile);
        }
        if name == RUN_SKILL_SCRIPT && self.node_skills(node).iter().any(|s| s.run_scripts) {
            return Some(Target::SkillScript);
        }
        if r.tools.iter().any(|t| t == name) {
            return self.resources.tools.get(name).map(Target::Tool);
        }
        let pool = self.mcp_pool()?;
        let server = r
            .mcp_servers
            .iter()
            .find(|s| pool.tools(s).iter().any(|t| t.name == name))?;
        self.resources.mcp.get(server).map(|server| Target::Mcp {
            server,
            tool: name.to_string(),
        })
    }

    /// Calls the model until it stops asking for tools. Each tool, and each MCP server, may be
    /// called at most its `max_iter` times in this run of the node, and the model may ask for tools in at most
    /// `max_rounds` rounds; asking once more is an error.
    /// `usage` and the time of the final response are the sum of all the calls; its `logprobs` are those of
    /// all the calls, in order.
    pub(super) async fn invoke_with_tools(
        &self,
        node: Entity,
        llm: &dyn Llm,
        mut call: ModelInvocation,
    ) -> Result<ModelResponse, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let mut used: HashMap<String, u32> = HashMap::new();
        let mut activated: std::collections::HashSet<String> = std::collections::HashSet::new();
        let (mut usage, mut time, mut logprobs) = (Default::default(), 0.0, Vec::new());
        let max_rounds = self
            .nodes
            .resources()
            .get(node)
            .and_then(|r| r.max_rounds)
            .unwrap_or(DEFAULT_MAX_ROUNDS);
        let mut rounds = 0u32;
        // `schema_prompt`: the attempts at an answer that is one JSON value, the first one included
        let (prompt, max_attempts, mut attempts) = (self.schema_prompt_of(node), self.max_attempts_of(node), 0u32);

        loop {
            let mut response = self
                .invoke_model(
                    &id,
                    self.nodes.model().get(node).map(|m| m.id.as_str()).unwrap_or_default(),
                    llm,
                    &call,
                )
                .await
                .map_err(|e| fail(Rule::ModelFailed, e.to_string()).with_detail_if(e.said()))?;
            usage += response.info.usage;
            time += response.info.execution_time;
            if let Some(why) = logprobs_missing(&call, &response) {
                return Err(fail(Rule::ModelFailed, why));
            }
            // the tokens of every call, in order: the last list is the final answer
            logprobs.append(&mut response.logprobs);
            if response.tools_call.is_empty() {
                // the final answer is the one that gives the JSON of the schema
                let model = self.nodes.model().get(node).map(|m| m.id.as_str()).unwrap_or_default();
                let mut done = match (&call.schema, &prompt) {
                    (Some(schema), _) => self.structured(&id, model, schema, SchemaOf::Node, response)?,
                    (None, Some(schema)) => {
                        attempts += 1;
                        let mechanism = Mechanism::Prompt {
                            attempt: attempts,
                            max: max_attempts,
                        };
                        match self.answer(&id, model, schema, mechanism, response)? {
                            Next::Done(r) => *r,
                            // the corrections stay in the call: a tool asked after one keeps them in its rounds
                            Next::Correct(messages) => {
                                call.corrections.extend(messages);
                                continue;
                            }
                        }
                    }
                    (None, None) => response,
                };
                done.info.usage = usage;
                done.info.execution_time = time;
                done.logprobs = logprobs;
                return Ok(done);
            }

            // every round counts, whatever the model asks for: a skill, a tool the node does not have, a wrong name
            rounds += 1;
            if rounds > max_rounds {
                return Err(fail(
                    Rule::ToolLimit,
                    format!("the model asked for tools in more than {max_rounds} rounds, the `max_rounds` of the node"),
                ));
            }

            // the cap of each tool and of each server, before anything is called
            let mut targets = Vec::with_capacity(response.tools_call.len());
            for c in &response.tools_call {
                let target = self.target(node, &c.function_name);
                let (name, max_iter) = match &target {
                    Some(Target::Tool(t)) => (Some(format!("tool `{}`", t.id)), t.max_iter),
                    Some(Target::Mcp { server, .. }) => (Some(format!("MCP server `{}`", server.id)), server.max_iter),
                    // a skill is given once; reading its files and running its scripts count together, for the skill
                    Some(Target::SkillFile | Target::SkillScript) => {
                        let skill = c
                            .arguments
                            .get("skill")
                            .and_then(|s| s.as_str())
                            .and_then(|n| self.node_skills(node).into_iter().find(|s| s.name == n));
                        match skill {
                            Some(s) => (Some(format!("skill `{}`", s.name)), s.max_iter),
                            None => (None, 0),
                        }
                    }
                    Some(Target::Skill) | None => (None, 0),
                };
                if let Some(name) = name {
                    let n = used.entry(name.clone()).or_insert(0);
                    *n += 1;
                    if *n > max_iter {
                        return Err(fail(
                            Rule::ToolLimit,
                            format!("the model asked for {name} more than {max_iter} times, its `max_iter`"),
                        ));
                    }
                }
                targets.push(target);
            }

            // the calls of one round go together; a tool the node does not have is told to the model
            let pool = self.mcp_pool();
            let skills = self.node_skills(node);
            // each skill is given once in a run of the node: the second time the model is told so
            let skill_answers: Vec<Option<Result<ToolOutcome, String>>> = response
                .tools_call
                .iter()
                .zip(&targets)
                .map(|(c, target)| match target {
                    Some(Target::Skill) => Some(activate(
                        &skills,
                        c.arguments.get("name").and_then(|n| n.as_str()),
                        &mut activated,
                    )),
                    Some(Target::SkillFile) => Some(Ok(read_skill_file(
                        &skills,
                        c.arguments.get("skill").and_then(|n| n.as_str()),
                        c.arguments.get("path").and_then(|n| n.as_str()),
                    ))),
                    _ => None,
                })
                .collect();
            let outcomes = join_all(response.tools_call.iter().zip(&targets).map(|(c, target)| {
                let pool = pool.clone();
                let skills = &skills;
                async move {
                    let started = std::time::Instant::now();
                    let invalid = match target {
                        Some(Target::Tool(t)) => serde_json::to_value(&t.parameters)
                            .ok()
                            .and_then(|schema| bad_arguments(&schema, &c.arguments)),
                        Some(Target::Mcp { server, tool }) => pool
                            .as_ref()
                            .and_then(|p| p.tools(&server.id).iter().find(|d| &d.name == tool))
                            .and_then(|d| bad_arguments(&d.parameters, &c.arguments)),
                        _ => None,
                    };
                    let outcome = match target {
                        _ if invalid.is_some() => Ok(ToolOutcome::failed(format!(
                            "the arguments do not match the parameters of `{}`: {}",
                            c.function_name,
                            invalid.unwrap_or_default()
                        ))),
                        Some(Target::Skill | Target::SkillFile) => Ok(ToolOutcome::ok(serde_json::Value::Null)),
                        Some(Target::SkillScript) => Ok(run_skill_script(skills, &c.arguments).await),
                        Some(Target::Tool(t)) => gate::call_tool(t, &c.arguments)
                            .await
                            .map_err(|e| format!("tool `{}`: {e}", t.id)),
                        Some(Target::Mcp { server, tool }) => match pool {
                            Some(pool) => pool.call(&server.id, tool, &c.arguments).await,
                            None => Err(format!("MCP server `{}` is not connected", server.id)),
                        },
                        None => Ok(ToolOutcome::failed(format!(
                            "this node has no tool called `{}`",
                            c.function_name
                        ))),
                    };
                    (outcome, started.elapsed().as_secs_f64())
                }
            }))
            .await;
            let mut results = Vec::with_capacity(outcomes.len());
            for (((c, (outcome, secs)), skill), target) in response
                .tools_call
                .iter()
                .zip(outcomes)
                .zip(skill_answers)
                .zip(&targets)
            {
                let outcome = match skill {
                    Some(answer) => answer.map_err(|e| fail(Rule::Io, e))?,
                    None => outcome.map_err(|e| fail(Rule::Io, e))?,
                };
                let kind = match target {
                    Some(Target::Mcp { .. }) => "mcp",
                    Some(Target::Skill | Target::SkillFile | Target::SkillScript) => "skill",
                    _ => "tool",
                };
                let result = match &outcome.result {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                self.emit(
                    self.traced(
                        Event::new(LogLevel::Debug, "tool.call")
                            .node(&id)
                            .field("kind", kind)
                            .field("name", c.function_name.clone())
                            .field("duration", secs)
                            .field("is_error", outcome.is_error)
                            .content("result", result),
                    ),
                );
                results.push(ToolResult {
                    call_id: c.call_id.clone(),
                    result: outcome.result,
                    is_error: outcome.is_error,
                });
            }
            call.tool_rounds.push(ToolRound {
                message: response.message,
                reasoning: response.reasoning,
                calls: response.tools_call,
                results,
            });
        }
    }
}

/// Why `arguments` do not satisfy the JSON Schema of a tool, or `None` if they do. A schema that
/// cannot be compiled is not enforced: the tool or the server that wrote it answers for it.
fn bad_arguments(schema: &serde_json::Value, arguments: &serde_json::Value) -> Option<String> {
    let validator = jsonschema::validator_for(schema).ok()?;
    let reasons: Vec<String> = validator
        .iter_errors(arguments)
        .map(|e| format!("{e} (at `{}`)", e.instance_path()))
        .collect();
    (!reasons.is_empty()).then(|| reasons.join("; "))
}

/// The answer to the activation of the skill `name`: its instructions in a tag with its name, or a
/// note that it is already in the conversation. A SKILL.md that cannot be read is an `Err`: it is a
/// mistake of the configuration, not something the model can fix.
fn activate(
    skills: &[&crate::resources::Skill],
    name: Option<&str>,
    activated: &mut std::collections::HashSet<String>,
) -> Result<ToolOutcome, String> {
    let Some(skill) = name.and_then(|n| skills.iter().find(|s| s.name == n)) else {
        return Ok(ToolOutcome::failed(format!(
            "this node has no skill called `{}`",
            name.unwrap_or_default()
        )));
    };
    if !activated.insert(skill.name.clone()) {
        return Ok(ToolOutcome::ok(serde_json::Value::String(format!(
            "the skill `{}` is already loaded above",
            skill.name
        ))));
    }
    let body = skill
        .instructions()
        .map_err(|e| format!("skill `{}`: cannot read its instructions: {e}", skill.name))?;
    // the files of the folder are only listed: the model reads the ones it needs
    let (files, truncated) = skill.files();
    let mut text = format!("<skill_content name=\"{}\">\n{}\n", skill.name, body.trim());
    if !files.is_empty() {
        text.push_str(&format!("\nThe paths below are relative to the folder of the skill; read a file with the {READ_SKILL_FILE} tool.\n<skill_resources>\n"));
        for f in &files {
            text.push_str(&format!("  <file>{f}</file>\n"));
        }
        if truncated {
            text.push_str("  <!-- the list is incomplete -->\n");
        }
        text.push_str("</skill_resources>\n");
    }
    text.push_str("</skill_content>");
    Ok(ToolOutcome::ok(serde_json::Value::String(text)))
}

/// The answer to a read of a file of the skill `skill`: its text, or why it cannot be read.
fn read_skill_file(skills: &[&crate::resources::Skill], skill: Option<&str>, path: Option<&str>) -> ToolOutcome {
    let Some(found) = skill.and_then(|n| skills.iter().find(|s| s.name == n)) else {
        return ToolOutcome::failed(format!("this node has no skill called `{}`", skill.unwrap_or_default()));
    };
    match found.read_file(path.unwrap_or_default()) {
        Ok(text) => ToolOutcome::ok(serde_json::Value::String(text)),
        Err(why) => ToolOutcome::failed(why),
    }
}

/// The answer to a run of a script of a skill: what it wrote, or why it could not be run.
async fn run_skill_script(skills: &[&crate::resources::Skill], arguments: &serde_json::Value) -> ToolOutcome {
    let name = arguments.get("skill").and_then(|n| n.as_str());
    let Some(skill) = name.and_then(|n| skills.iter().find(|s| s.name == n)) else {
        return ToolOutcome::failed(format!("this node has no skill called `{}`", name.unwrap_or_default()));
    };
    if !skill.run_scripts {
        return ToolOutcome::failed(format!("the scripts of the skill `{}` cannot be run", skill.name));
    }
    let script = match skill.script_path(arguments.get("path").and_then(|p| p.as_str()).unwrap_or_default()) {
        Ok(p) => p,
        Err(why) => return ToolOutcome::failed(why),
    };
    let args: Vec<String> = arguments
        .get("args")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    gate::run_script(&skill.path, &script, &args, skill.timeout, skill.max_file_bytes).await
}
