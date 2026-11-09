//! Running the graph: the list of the edges is read in order and each node is run with `run_node`.
//! What a node sends to the connected node is the `agent_message` of the next one; what it sends to
//! the final output is added to the list that `run` returns. A group runs its `in`, then its
//! branches, and hands the answers of the branches, packed as one, to its `out`:
//! `linear` runs the branches one after the other (a `for` loop), `parallel` launches all of them
//! together, each as a task of its own that the runtime spreads over its threads, `concurrent` does
//! the same through a semaphore of `limit` permits, `batch` sends the requests of all the branches
//! to the provider as one batch job (or several, of `limit` requests each). A `react` makes its
//! controller answer a routing output at every turn until it says it is done. A validation node
//! lets the message it examined pass unchanged when the judge approves; when it refuses, either the
//! run ends there (`stop`) or the problem goes to the next node (`node`).

use std::sync::Arc;

use futures_util::future::{BoxFuture, FutureExt};
use serde_json::{Value, json};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::events::{CTX, Where, here};
use super::judge::Verdict;
use super::{Content, Delivery, NodeInput, NodeOutput, Program};
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;
use crate::graph::edges::{Group, Pattern, React, Step};
use crate::graph::nodes::component::OnError;
use crate::models::{BatchRequest, ChatEntry, MediaOutput, Role};
use crate::schema::key;

/// What a node sent to the final output.
#[derive(Debug, Clone)]
pub struct OutputItem {
    pub agent_id: String,
    pub message: Option<Content>,
    pub media: Vec<MediaOutput>,
    pub reasoning: Vec<Value>,
}

/// The final output: the items in the order of the edges.
#[derive(Debug, Clone, Default)]
pub struct RunOutput {
    pub items: Vec<OutputItem>,
}

/// What is handed from a node (or a group) to the next one.
#[derive(Debug, Clone, Default)]
struct Carry {
    message: Option<String>,
    media: Vec<MediaOutput>,
}

impl Carry {
    fn of(d: &Delivery) -> Carry {
        Carry {
            message: d.message.as_ref().map(text_of),
            media: d.media.clone(),
        }
    }
}

/// The text a message becomes when it is passed on; JSON is written out.
fn text_of(c: &Content) -> String {
    match c {
        Content::Text(t) => t.clone(),
        Content::Json(j) => serde_json::to_string_pretty(j).unwrap_or_else(|_| j.to_string()),
    }
}

/// What a series of steps gave: the last thing handed on, and what went to the final output.
struct Stepped {
    carry: Carry,
    items: Vec<OutputItem>,
    /// A judge ended the run. A judge stands in the main flow only, so only there is it noticed.
    halted: bool,
}

/// What one node gave: what it sends on, what it sends to the final output, and whether it ended the run.
struct Ran {
    delivery: Delivery,
    item: Option<OutputItem>,
    halted: bool,
}

/// The answer of one branch of a group.
struct Branch {
    /// The node the answer comes from.
    origin: String,
    delivery: Delivery,
    items: Vec<OutputItem>,
}

fn fail(rule: Rule, msg: impl Into<String>) -> Error {
    Error::new(
        Phase::Execution,
        Path::root().key(key::GRAPH).key(key::EDGES),
        rule,
        msg,
    )
}

impl Program {
    /// Runs the graph with the message of the user and returns the final output. The program is in
    /// an `Arc` because the branches of a `parallel` group are tasks that outlive a plain reference.
    /// On a multi-thread runtime they run on different threads.
    pub async fn run(self: &Arc<Self>, user_message: &str) -> Result<RunOutput, Error> {
        self.run_until(user_message, std::future::pending()).await
    }

    /// Runs the graph as `run` does, but the nodes are stopped when `stop` is ready before they end, and its error
    /// is the outcome. Either way the run ends as always: its MCP servers are closed, its last event is logged and
    /// its trace is sent. The service stops so a run that lasts too long, or that is going on when it shuts down.
    pub async fn run_until(
        self: &Arc<Self>,
        user_message: &str,
        stop: impl Future<Output = Error>,
    ) -> Result<RunOutput, Error> {
        self.run_started();
        let started = std::time::Instant::now();
        // the MCP servers are connected before the first node and closed after the last, or after the error
        let nodes = async {
            self.connect_mcp().await?;
            self.run_nodes(user_message).await
        };
        // a stop drops the nodes halfway: the tasks of their branches are aborted (`spawn_branches`)
        let output = tokio::select! {
            output = nodes => output,
            e = stop => Err(e),
        };
        self.close_mcp().await;
        self.run_finished(started, output.as_ref().err());
        // the timeline of the run goes to the collector, whatever the outcome
        if let Some(t) = &self.tracer {
            t.flush().await;
        }
        output
    }

    async fn run_nodes(self: &Arc<Self>, user_message: &str) -> Result<RunOutput, Error> {
        let user = user_message.to_string();
        let mut items = Vec::new();

        let first = self.step_node(self.edges.start(), &Carry::default(), &user).await?;
        items.extend(first.item);
        if first.halted {
            return Ok(RunOutput { items });
        }
        let stepped = self
            .clone()
            .run_steps(self.edges.steps().to_vec(), Carry::of(&first.delivery), user.clone())
            .await?;
        items.extend(stepped.items);
        // a judge that stops the run sends it straight to the output, `end` is not reached
        if stepped.halted {
            return Ok(RunOutput { items });
        }
        // a graph of one node has no `end`: its only node has run
        if let Some(end) = self.edges.end() {
            let last = self.step_node(end, &stepped.carry, &user).await?;
            items.extend(last.item);
        }
        Ok(RunOutput { items })
    }

    /// Runs one node: what it sends on, and what it sends to the final output.
    async fn step_node(&self, node: Entity, carry: &Carry, user: &str) -> Result<Ran, Error> {
        let input = NodeInput {
            user_message: Some(user.to_string()),
            agent_message: carry.message.clone(),
            agent_media: carry.media.clone(),
        };
        if let Some(v) = self.nodes.validation().get(node) {
            return self.step_judge(node, v.on_error, carry, &input).await;
        }
        let out = self.run_node(node, input).await?;
        let (delivery, item) = self.parts(node, out);
        Ok(Ran {
            delivery,
            item,
            halted: false,
        })
    }

    /// A validation node: approved, the message it examined goes on unchanged (only the message);
    /// refused, the run ends and the final output carries the reasons (`stop`) or the problem is the message of the next node (`node`).
    async fn step_judge(
        &self,
        node: Entity,
        on_error: OnError,
        carry: &Carry,
        input: &NodeInput,
    ) -> Result<Ran, Error> {
        let id = self.nodes.id(node).unwrap_or_default();
        let message = |text: Option<String>| Delivery {
            message: text.map(Content::Text),
            ..Delivery::default()
        };
        Ok(match (self.judge(node, input).await?, on_error) {
            (Verdict::Approved, _) => Ran {
                delivery: message(carry.message.clone()),
                item: None,
                halted: false,
            },
            (Verdict::Refused { reasons, log }, OnError::Stop) => {
                // the final output says why the run ended
                let why = OutputItem {
                    agent_id: id.to_string(),
                    message: Some(Content::Text(format!("reasons: {reasons}\nlog: {log}"))),
                    media: Vec::new(),
                    reasoning: Vec::new(),
                };
                Ran {
                    delivery: Delivery::default(),
                    item: Some(why),
                    halted: true,
                }
            }
            (Verdict::Refused { reasons, log }, OnError::Node) => Ran {
                delivery: message(Some(format!(
                    "---\nfrom agent {id}\n---\nreasons: {reasons}\nlog: {log}"
                ))),
                item: None,
                halted: false,
            },
        })
    }

    /// What a node sent on, and what it sent to the final output.
    fn parts(&self, node: Entity, out: NodeOutput) -> (Delivery, Option<OutputItem>) {
        let a = out.assistant;
        let item = (a.message.is_some() || !a.media.is_empty() || !a.reasoning.is_empty()).then(|| OutputItem {
            agent_id: self.nodes.id(node).unwrap_or_default().to_string(),
            message: a.message,
            media: a.media,
            reasoning: a.reasoning,
        });
        (out.node, item)
    }

    fn run_steps(
        self: Arc<Self>,
        steps: Vec<Step>,
        carry: Carry,
        user: String,
    ) -> BoxFuture<'static, Result<Stepped, Error>> {
        async move {
            let (mut carry, mut items) = (carry, Vec::new());
            for step in steps {
                match step {
                    Step::Node(x) => {
                        let ran = self.step_node(x, &carry, &user).await?;
                        items.extend(ran.item);
                        if ran.halted {
                            return Ok(Stepped {
                                carry,
                                items,
                                halted: true,
                            });
                        }
                        carry = Carry::of(&ran.delivery);
                    }
                    Step::Group(g) => {
                        let branch = self.clone().run_group(g, carry.clone(), user.clone()).await?;
                        items.extend(branch.items);
                        carry = Carry::of(&branch.delivery);
                    }
                    Step::React(r) => {
                        let branch = self.clone().run_react(r, carry.clone(), user.clone()).await?;
                        items.extend(branch.items);
                        carry = Carry::of(&branch.delivery);
                    }
                }
            }
            Ok(Stepped {
                carry,
                items,
                halted: false,
            })
        }
        .boxed()
    }

    /// A group: its `in`, then the branches, then its `out` with the answers of the branches packed.
    fn run_group(self: Arc<Self>, g: Group, carry: Carry, user: String) -> BoxFuture<'static, Result<Branch, Error>> {
        async move {
            let pattern = format!("{:?}", g.pattern).to_lowercase();
            let (started, span) = self.group_started(&pattern, g.fan.len());
            let inside = Where {
                depth: here().depth + 1,
                span,
            };
            let branch = CTX.scope(inside, self.clone().group_body(g, carry, user)).await?;
            self.group_finished(&pattern, started, span);
            Ok(branch)
        }
        .boxed()
    }

    fn group_body(self: Arc<Self>, g: Group, carry: Carry, user: String) -> BoxFuture<'static, Result<Branch, Error>> {
        async move {
            let mut items = Vec::new();
            let origin_in = self.nodes.id(g.r#in).unwrap_or_default().to_string();
            let ran = self.step_node(g.r#in, &carry, &user).await?;
            items.extend(ran.item);
            let input = Carry::of(&ran.delivery);

            let branches = match g.pattern {
                Pattern::Linear => {
                    let mut done = Vec::with_capacity(g.fan.len());
                    for step in g.fan.iter().cloned() {
                        done.push(self.clone().run_branch(step, input.clone(), user.clone()).await?);
                    }
                    done
                }
                Pattern::Parallel | Pattern::Concurrent => self.spawn_branches(&g, &input, &user).await?,
                Pattern::Batch => self.run_batch(&g, &input, &user).await?,
            };
            // the answers go to the final output in the order of the fan
            for b in &branches {
                items.extend(b.items.iter().cloned());
            }
            let packed = pack(&branches)?;
            match g.out {
                Some(out) => {
                    let ran = self.step_node(out, &packed, &user).await?;
                    items.extend(ran.item);
                    Ok(Branch {
                        origin: self.nodes.id(out).unwrap_or_default().to_string(),
                        delivery: ran.delivery,
                        items,
                    })
                }
                // without `out` nothing is passed on; what follows starts when the branches are done
                None => Ok(Branch {
                    origin: origin_in,
                    delivery: Delivery::default(),
                    items,
                }),
            }
        }
        .boxed()
    }

    /// The branches of a `batch` group as one batch job: the requests of all the nodes go together
    /// (`limit` at a time, if there is a limit) and the answers come back to the nodes. The nodes
    /// of a batch use the same model, which is the one that runs the job.
    async fn run_batch(self: &Arc<Self>, g: &Group, input: &Carry, user: &str) -> Result<Vec<Branch>, Error> {
        let nodes: Vec<Entity> = g
            .fan
            .iter()
            .map(|s| match s {
                Step::Node(x) => Ok(*x),
                _ => Err(fail(
                    Rule::NotSupported,
                    "the branches of a `batch` group are plain nodes",
                )),
            })
            .collect::<Result<_, _>>()?;
        let node_input = NodeInput {
            user_message: Some(user.to_string()),
            agent_message: input.message.clone(),
            agent_media: input.media.clone(),
        };
        let mut prepared = Vec::new();
        for n in &nodes {
            prepared.push(self.prepare_node(*n, &node_input).await?);
        }
        let node_error = |i: usize, msg: String| {
            Error::new(
                Phase::Execution,
                Path::node(self.nodes.id(nodes[i]).unwrap_or_default()),
                Rule::ModelFailed,
                msg,
            )
        };

        let Some(first) = prepared.first() else {
            return Ok(Vec::new());
        };
        let llm = self
            .models
            .shared(&first.model)
            .ok_or_else(|| node_error(0, format!("model `{}` is not built", first.model)))?;
        let size = g.limit.map_or(nodes.len(), |l| (l as usize).max(1));
        let mut answers: Vec<Option<crate::models::ModelResponse>> = (0..nodes.len()).map(|_| None).collect();
        let positions: Vec<usize> = (0..nodes.len()).collect();
        for chunk in positions.chunks(size) {
            let requests = chunk
                .iter()
                .map(|&i| BatchRequest {
                    id: i.to_string(),
                    invocation: prepared[i].call.clone(),
                })
                .collect();
            // the state of the job is told through this channel: until the job has ended, letting go of it here
            // cancels it at the provider. The call is a task of its own, so that a job being created when the run
            // lets go of it is still created, and then cancelled (`OpenBatch`)
            let (events, heard) = tokio::sync::mpsc::unbounded_channel();
            let call = tokio::spawn({
                let llm = llm.clone();
                async move { llm.invoke_batch(requests, events).await }
            });
            let mut open = super::batch::OpenBatch::new(heard, call, llm.clone(), &self.cancels);
            let results = open
                .outcome()
                .await
                .map_err(|e| node_error(chunk[0], format!("the batch stopped: {e}")))?
                .map_err(|e| node_error(chunk[0], e.to_string()).with_detail_if(e.said()))?;
            for r in results {
                let i: usize = r
                    .id
                    .parse()
                    .map_err(|_| node_error(chunk[0], format!("the batch answered an unknown request `{}`", r.id)))?;
                answers[i] = Some(
                    r.result
                        .map_err(|e| node_error(i, e.to_string()).with_detail_if(e.said()))?,
                );
            }
        }
        let mut done = Vec::with_capacity(nodes.len());
        for (i, node) in nodes.iter().enumerate() {
            let response = answers[i]
                .take()
                .ok_or_else(|| node_error(i, "the batch gave no answer for this node".to_string()))?;
            let (delivery, item) = self.parts(*node, self.finish_node(*node, &prepared[i].input, response).await?);
            done.push(Branch {
                origin: self.nodes.id(*node).unwrap_or_default().to_string(),
                delivery,
                items: item.into_iter().collect(),
            });
        }
        Ok(done)
    }

    /// Every branch is a task of its own; a semaphore lets `limit` of them in at a time (`parallel`
    /// without a limit lets them all in). The branches are taken as they end, so the first one that fails
    /// ends the group at once; the answers come back in the order of the fan.
    ///
    /// The tasks are stopped when `tasks` is dropped: when a branch fails, and when the nodes of the run are
    /// dropped (the service stops a run, see `run_until`), so no branch goes on alone.
    async fn spawn_branches(self: &Arc<Self>, g: &Group, input: &Carry, user: &str) -> Result<Vec<Branch>, Error> {
        let permits = g.limit.map_or(g.fan.len(), |l| l as usize).clamp(1, g.fan.len().max(1));
        let gate = Arc::new(Semaphore::new(permits));
        let mut tasks = JoinSet::new();
        for (i, step) in g.fan.iter().cloned().enumerate() {
            let (this, gate, input, user) = (self.clone(), gate.clone(), input.clone(), user.to_string());
            tasks.spawn(CTX.scope(here(), async move {
                let _permit = gate.acquire_owned().await.map_err(|e| fail(Rule::Io, e.to_string()))?;
                this.run_branch(step, input, user).await.map(|b| (i, b))
            }));
        }
        let mut done: Vec<Option<Branch>> = (0..g.fan.len()).map(|_| None).collect();
        while let Some(joined) = tasks.join_next().await {
            // the first error ends the run: the branches still going are stopped when `tasks` is dropped
            let (i, b) = joined.map_err(|e| fail(Rule::Io, format!("a branch stopped: {e}")))??;
            done[i] = Some(b);
        }
        Ok(done.into_iter().flatten().collect())
    }

    fn run_branch(
        self: Arc<Self>,
        step: Step,
        input: Carry,
        user: String,
    ) -> BoxFuture<'static, Result<Branch, Error>> {
        async move {
            match step {
                Step::Node(x) => {
                    let ran = self.step_node(x, &input, &user).await?;
                    Ok(Branch {
                        origin: self.nodes.id(x).unwrap_or_default().to_string(),
                        delivery: ran.delivery,
                        items: ran.item.into_iter().collect(),
                    })
                }
                Step::Group(g) => self.run_group(g, input, user).await,
                Step::React(r) => self.run_react(r, input, user).await,
            }
        }
        .boxed()
    }

    /// A `react`: the controller answers the routing output at every turn. If it is not done, the
    /// worker it names runs with the `input` it wrote, and what the worker sends on comes back to
    /// the controller as an observation. The controller is the only link with the rest of the graph.
    fn run_react(self: Arc<Self>, r: React, carry: Carry, user: String) -> BoxFuture<'static, Result<Branch, Error>> {
        async move {
            let (started, span) = self.group_started("react", r.workers.len());
            let inside = Where {
                depth: here().depth + 1,
                span,
            };
            let branch = CTX.scope(inside, self.clone().react_body(r, carry, user)).await?;
            self.group_finished("react", started, span);
            Ok(branch)
        }
        .boxed()
    }

    fn react_body(self: Arc<Self>, r: React, carry: Carry, user: String) -> BoxFuture<'static, Result<Branch, Error>> {
        async move {
            let controller = r.controller;
            let cid = self.nodes.id(controller).unwrap_or_default().to_string();
            let error = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&cid), rule, msg);

            let ids = r.worker_ids(&self.nodes);
            let node_input = NodeInput {
                user_message: Some(user.clone()),
                agent_message: carry.message.clone(),
                agent_media: carry.media.clone(),
            };
            let prepared = self.prepare_node(controller, &node_input).await?;
            let llm = self
                .models
                .get(&prepared.model)
                .ok_or_else(|| error(Rule::UnknownRef, format!("model `{}` is not built", prepared.model)))?;
            let mut call = prepared.call;
            call.schema = Some(React::routing_schema(&ids));
            // what the controller reads from its memory stays in every turn, before the steps of the react
            let remembered = call.history.clone();

            let (mut turns, mut items, mut dispatched) = (Vec::new(), Vec::new(), 0u32);
            // the controller ends with the usage and the time of all its turns, as a node with tools does
            let (mut usage, mut time) = (Default::default(), 0.0);
            loop {
                let response = llm
                    .invoke(&call)
                    .await
                    .map_err(|e| error(Rule::ModelFailed, e.to_string()).with_detail_if(e.said()))?;
                self.model_called(&cid, &prepared.model, &call, &response);
                usage += response.info.usage;
                time += response.info.execution_time;
                let routing = response.structured.clone().filter(Value::is_object).ok_or_else(|| {
                    error(
                        Rule::BadValue,
                        "the controller did not answer with the routing output".into(),
                    )
                })?;
                let done = routing["done"]
                    .as_bool()
                    .ok_or_else(|| error(Rule::BadValue, "the routing output has no `done`".into()))?;

                if done {
                    let answer = routing["answer"].as_str().filter(|a| !a.is_empty()).ok_or_else(|| {
                        error(Rule::MissingField, "the controller is done and gave no `answer`".into())
                            .with_detail(routing.to_string())
                    })?;
                    let mut last = response;
                    (last.message, last.structured) = (Some(answer.to_string()), None);
                    (last.info.usage, last.info.execution_time) = (usage, time);
                    let (delivery, item) =
                        self.parts(controller, self.finish_node(controller, &prepared.input, last).await?);
                    items.extend(item);
                    return Ok(Branch {
                        origin: cid.clone(),
                        delivery,
                        items,
                    });
                }

                if dispatched >= r.max_iterations {
                    return Err(error(
                        Rule::ReactLimit,
                        format!(
                            "the controller asked for a worker after {} calls, the limit of `max_iterations`",
                            r.max_iterations
                        ),
                    ));
                }
                let name = routing["next_worker"]
                    .as_str()
                    .filter(|n| !n.is_empty())
                    .ok_or_else(|| {
                        error(
                            Rule::MissingField,
                            "the controller is not done and named no `next_worker`".into(),
                        )
                        .with_detail(routing.to_string())
                    })?
                    .to_string();
                let position = ids.iter().position(|i| *i == name).ok_or_else(|| {
                    error(
                        Rule::UnknownRef,
                        "the controller named a worker that is not in this react".into(),
                    )
                    .with_detail(name.clone())
                })?;
                dispatched += 1;

                let handed = Carry {
                    message: Some(routing["input"].as_str().unwrap_or_default().to_string()),
                    media: Vec::new(),
                };
                let worked = match r.workers[position].clone() {
                    Step::Node(x) => {
                        let ran = self.step_node(x, &handed, &user).await?;
                        Branch {
                            origin: name.clone(),
                            delivery: ran.delivery,
                            items: ran.item.into_iter().collect(),
                        }
                    }
                    Step::Group(g) => self.clone().run_group(g, handed, user.clone()).await?,
                    // the loader refuses a `react` among the workers (`react_nested`)
                    Step::React(_) => {
                        return Err(error(
                            Rule::ReactNested,
                            "a `react` cannot be a worker of another `react`".into(),
                        ));
                    }
                };
                items.extend(worked.items);

                // the next turn: what was said so far, and the observation
                let observed = Carry::of(&worked.delivery)
                    .message
                    .unwrap_or_else(|| "(no output)".to_string());
                turns.push(ChatEntry {
                    role: Role::User,
                    content: call.user.clone(),
                });
                turns.push(ChatEntry {
                    role: Role::Assistant,
                    content: routing.to_string(),
                });
                call.history = remembered.iter().cloned().chain(turns.iter().cloned()).collect();
                call.user = format!("Observation from {name}: {observed}");
                call.media.clear();
            }
        }
        .boxed()
    }
}

/// The answers of the branches as one: texts one after the other, each preceded by its origin; or,
/// for structured answers, a list of `{ from_node, content }`.
fn pack(branches: &[Branch]) -> Result<Carry, Error> {
    let mut texts = Vec::new();
    let mut jsons = Vec::new();
    let mut media = Vec::new();
    for b in branches {
        match &b.delivery.message {
            Some(Content::Text(t)) => texts.push(format!("---\nfrom agent {}\n---\n{}", b.origin, t)),
            Some(Content::Json(j)) => jsons.push(json!({ "from_node": b.origin, "content": j })),
            None => {}
        }
        media.extend(b.delivery.media.iter().cloned());
    }
    let message = match (texts.is_empty(), jsons.is_empty()) {
        (false, false) => {
            return Err(fail(
                Rule::NotSupported,
                "the branches of a group give both text and structured output",
            ));
        }
        (false, true) => Some(texts.join("\n\n")),
        (true, false) => Some(serde_json::to_string_pretty(&Value::Array(jsons)).unwrap_or_default()),
        (true, true) => None,
    };
    Ok(Carry { message, media })
}
