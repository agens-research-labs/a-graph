# Edges

`graph.edges` says in which order the nodes run. It is a list read in order, like a program:

```yaml
edges:
  - start: translator          # the first node
  - parallel:                  # a group
      - in: framing
      - fan:
          - next: advantages
          - next: disadvantages
      - out: balance
  - end: closing               # the last node
```

Every entry is a map with one key. Every key is described in [the reference](graph.md).

## The main flow

- `start` is the first entry, and there is exactly one (`edge_start`).
- `next` runs a node after the one before it.
- `end` is the last entry, and there is exactly one (`edge_end`).
- The groups (`linear`, `parallel`, `concurrent`, `batch`) and `react` are entries of the list like `next`.

A graph of one node is the `start` alone, with nothing else: the node runs once and its output is the result. Every
other graph needs its `end`.

A node may appear more than once in the edges, and each appearance is a run of it. `start: a` and `end: a` is an edge
from a node to itself: `a` runs twice.

What a node sends on (its `message.output.node` and the media with `node: true`) is what the next entry receives as
its `agent_message` and its media. See [nodes](nodes.md).

## Groups

`linear`, `parallel`, `concurrent` and `batch` have the same shape: a list of `limit`, `in`, `fan` and `out`, each
written once and in this order (`block_shape`, `exactly_one`).

- `in` (mandatory): the node that runs first. What it sends on is the input of every branch.
- `fan` (mandatory, at least one branch): the branches. A branch is a node (`next: <id>`) or a whole group or `react`,
  so groups can be put inside groups.
- `out` (optional): the node that receives the answers of all the branches, packed as one, and whose output goes on to
  the rest of the graph. A group may have no `out` only if none of its nodes sends its output on
  (`group_out_required`); then the group sends nothing on, and the next entry starts when the branches are done.
- `limit`: how many branches run at the same time.

The four groups differ only in how the branches run:

| Group | How the branches run | `limit` |
|---|---|---|
| `linear` | one after the other | not allowed |
| `parallel` | all together, each as a task of its own | optional: at most `limit` at a time |
| `concurrent` | all together, at most `limit` at a time | mandatory |
| `batch` | as one batch job of the provider, instead of one call each (see [models](models.md#batch)) | optional: at most `limit` requests in each job |

The branches of a `batch` are plain nodes, all with the same model: a group or a `react` among them is an error.

In `parallel` and `concurrent`, the first branch that fails ends the run, and the branches still running are stopped.

### How the answers are packed

The `out` node receives the answers of the branches, in the order of the `fan`, as its `agent_message`:

- texts are joined one after the other, each preceded by the node it comes from:

  ```
  ---
  from agent advantages
  ---
  <the text>
  ```

- answers with an output `schema` become one JSON list of `{ "from_node": "<id>", "content": <the answer> }`;
- the media of all the branches are passed together.

Branches that give both texts and structured answers cannot be packed: the run fails.

What the nodes of a group send to the final output goes there in the order of the `fan`.

## React

A `react` is not a group of branches: a controller decides at run time which worker to call.

```yaml
- react:
    controller: planner
    max_iterations: 10
    components:
      - worker: researcher
      - worker: writer
```

At every turn the controller answers with a fixed structure, which the program asks of its model:

- `done`: `true` when it gives the final answer, `false` when it calls a worker;
- `next_worker`: the worker to call;
- `input`: what the worker receives, as its `agent_message`;
- `answer`: the final answer;
- `reasoning`: why it chose this step.

The worker runs, and what it sends on comes back to the controller as an observation for the next turn. When the
controller is done, its `answer` is the output of the `react`, delivered as the output of the controller. The
controller is the only link between the workers and the rest of the graph.

- A worker is a node (`worker: <id>`) or a whole group; a group is called by the id of its `in` node. Two workers with
  the same id are an error (`duplicate_id`).
- The program gives the controller the list of the worker ids; what each worker does must be said in the prompt of the
  controller.
- `max_iterations` is the most calls to workers. A controller that asks for one more fails the run (`react_limit`).
- The controller has no tools, MCP servers, skills, blackboard or output `schema` of its own: its output is defined
  by the `react`. Give those to a worker.
- A `react` inside a `react` cannot run yet.

## Roles

A node has no field that says its role: it is a controller or a worker because it appears as one in a `react`, and a
plain step anywhere else.
