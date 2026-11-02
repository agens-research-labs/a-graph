//! `graph.edges`: the order of execution. A list read in order: `start`, `next`, `end`, and the
//! groups (`linear`, `parallel`, `concurrent`, `batch`) and `react`, which are special `next`.
//! Everything can be nested: a branch of a `fan`, and a `worker` of a `react`, can be a whole
//! group. Which parts come in which order and what they mean is checked by the manager of the edges.

use super::key::*;
use super::{Field, Kind, Shape};

const NODE_REF: Shape = Shape::Ref(Kind::Node);

/// One entry of a group: `limit`, `in`, `fan` or `out`.
static GROUP_ITEM: Shape = Shape::Either(&[
    Shape::Map(&[Field::req(LIMIT, Shape::Int).doc("How many branches run at the same time; in `batch`, how many requests go in each job.")]),
    Shape::Map(&[Field::req(IN, NODE_REF).doc("The node that runs first. What it sends on is the input of every branch. Mandatory.")]),
    Shape::Map(&[Field::req(FAN, Shape::List(&BRANCH)).doc("The branches, at least one. Mandatory.")]),
    Shape::Map(&[Field::req(OUT, NODE_REF).doc(
        "The node that receives the answers of all the branches, packed as one, in the order of the `fan`, and whose output goes on. It may be left out only if no node of the group sends its output on (`group_out_required`).",
    )]),
]);

static GROUP_SHAPE: Shape = Shape::List(&GROUP_ITEM);

const GROUP: Shape = Shape::Lazy {
    name: "group",
    doc: "The entries of a group: `limit`, `in`, `fan` and `out`, each written once and in this order.",
    get: group,
};

static REACT_SHAPE: Shape = Shape::Map(&[
    Field::req(CONTROLLER, NODE_REF).doc(
        "The node that decides at every turn which worker to call, or gives the final answer, which is the output of the `react`. It has no tools, MCP servers, skills, blackboard or output `schema` of its own.",
    ),
    Field::req(MAX_ITERATIONS, Shape::Int).doc("The most calls to workers. One more fails the run (`react_limit`)."),
    Field::req(COMPONENTS, Shape::List(&WORKER_ENTRY)).doc("The workers the controller may call. Two workers with the same id are an error (`duplicate_id`)."),
]);

const REACT_BODY: Shape = Shape::Lazy {
    name: "react",
    doc: "A controller and the workers it calls.",
    get: react,
};

const NEXT_DOC: &str = "A node that runs after the entry before it. A node may appear more than once in the edges, and each appearance is a run of it.";

const LINEAR_GROUP: Field =
    Field::req(LINEAR, GROUP).doc("A group whose branches run one after the other. `limit` is not allowed.");
const PARALLEL_GROUP: Field = Field::req(PARALLEL, GROUP).doc(
    "A group whose branches run all together, each as a task of its own; with `limit`, at most that many at a time. The first branch that fails ends the run.",
);
const CONCURRENT_GROUP: Field = Field::req(CONCURRENT, GROUP).doc("A group whose branches run all together, at most `limit` at a time; `limit` is mandatory. The first branch that fails ends the run.");
const BATCH_GROUP: Field = Field::req(BATCH, GROUP).doc(
    "A group whose branches are sent to the provider as one batch job; with `limit`, at most that many requests in each job. The branches are plain nodes, all with the same model.",
);
const REACT_ENTRY: Field = Field::req(REACT, REACT_BODY)
    .doc("A controller that decides at run time which worker to call. A `react` inside a `react` cannot run yet.");

/// What can be a branch of a `fan`.
static BRANCH_SHAPE: Shape = Shape::Either(&[
    Shape::Map(&[Field::req(NEXT, NODE_REF).doc("A branch made of one node.")]),
    Shape::Map(&[LINEAR_GROUP]),
    Shape::Map(&[PARALLEL_GROUP]),
    Shape::Map(&[CONCURRENT_GROUP]),
    Shape::Map(&[BATCH_GROUP]),
    Shape::Map(&[REACT_ENTRY]),
]);

const BRANCH: Shape = Shape::Lazy {
    name: "branch",
    doc: "One branch of a `fan`: a node, or a whole group or `react`.",
    get: branch,
};

/// What can be a `worker` of a `react`.
static WORKER_SHAPE: Shape = Shape::Either(&[
    Shape::Map(&[Field::req(WORKER, NODE_REF).doc("A worker made of one node.")]),
    Shape::Map(&[LINEAR_GROUP]),
    Shape::Map(&[PARALLEL_GROUP]),
    Shape::Map(&[CONCURRENT_GROUP]),
    Shape::Map(&[BATCH_GROUP]),
    Shape::Map(&[REACT_ENTRY]),
]);

const WORKER_ENTRY: Shape = Shape::Lazy {
    name: "worker",
    doc: "One worker of a `react`: a node, or a whole group, called by the id of its `in` node.",
    get: worker,
};

/// One entry of the list.
static STEP_SHAPE: Shape = Shape::Either(&[
    Shape::Map(&[Field::req(START, NODE_REF).doc("The first node: the first entry, and the only `start` (`edge_start`). A graph of one node is its `start` alone.")]),
    Shape::Map(&[Field::req(NEXT, NODE_REF).doc(NEXT_DOC)]),
    Shape::Map(&[Field::req(END, NODE_REF).doc("The last node: the last entry, and the only `end` (`edge_end`). Every graph needs it, except one of a single node.")]),
    Shape::Map(&[LINEAR_GROUP]),
    Shape::Map(&[PARALLEL_GROUP]),
    Shape::Map(&[CONCURRENT_GROUP]),
    Shape::Map(&[BATCH_GROUP]),
    Shape::Map(&[REACT_ENTRY]),
]);

fn group() -> &'static Shape {
    &GROUP_SHAPE
}

fn react() -> &'static Shape {
    &REACT_SHAPE
}

fn branch() -> &'static Shape {
    &BRANCH_SHAPE
}

fn worker() -> &'static Shape {
    &WORKER_SHAPE
}

fn step() -> &'static Shape {
    &STEP_SHAPE
}

pub(super) const EDGES: Shape = Shape::List(&Shape::Lazy {
    name: "step",
    doc: "One entry of `graph.edges`.",
    get: step,
});
