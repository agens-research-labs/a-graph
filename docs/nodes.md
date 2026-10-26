# Nodes

A node is a model plus what it uses (a prompt, tools, media, a memory...), where its input comes from and where its
output goes. Nodes are declared in `graph.nodes` and connected in `graph.edges` (see [edges](edges.md)). Every key is
described in [the reference](graph.md).

## Text and media

What a node receives and produces is split by its nature:

- `message` is text: `message.input` says where the text comes from, `message.output` where it goes;
- `media` is everything that is not text: `media.input` lists the media the node reads, `media.output` the media it
  produces (see [media](media.md)).

Every source and every destination is a separate `true` or `false`, and any combination is valid. Several can be
`true` at the same time; all can be `false`, for a node that works only with its other resources, or that is useful
only for what it does along the way (writing a blackboard or a memory, calling tools), and whose output is discarded.

## Where the text comes from

- `user_message: true`: the message of the user, the one given to the command.
- `agent_message: true`: what the node connected before sends on. Which node that is, is decided by `edges`, never by
  an id written on the node.

A node that takes a message that does not arrive (for example `agent_message` after a node that sends nothing on)
fails when it runs. How the two messages reach the prompt is explained in [prompts](prompts.md).

## The run of a node

1. The incoming text goes through the utilities of `message.input.utility`, if any. The changed text is the one used
   from here on: in the prompt, in the queries to the vector databases and memories, and in what the memory saves.
2. The node reads what it declares: the vector databases (`{{chunks}}`), the memories (the history and the summary),
   the blackboard if it reads it, the media of `media.input`.
3. The placeholders of the prompt are filled, and the model is called.
4. If the model asks for tools, MCP tools or skills, they are called and the model is called again with their
   results, until it gives its final answer (see [tools](tools.md)). The loop has at most `resources.max_rounds` rounds
   (20 by default); one more fails the run. This loop is not part of `message`: from the point of view of the graph,
   the node gives only its final answer.
5. The answer is delivered.

## Where the answer goes

The text of the answer goes through the utilities of `message.output.utility` once, and then the same changed text
goes to every destination:

- `assistant_message: true`: to the final output (see [the command line](cli.md#the-result));
- `node: true`: to the node connected after, as its `agent_message`;
- the blackboard, if the node writes it, and the memories, according to their `write`.

`reasoning: true` adds the reasoning of the model to the final output. The reasoning never goes to another node.

With an output `schema` (a JSON Schema written on the node), the model answers in that shape and the answer is a JSON
value instead of a text. A node that has a `schema` and goes through output utilities gives them the JSON as a text,
and what they give back is a text.

Each produced media decides its own destinations with its `assistant` and `node`.

## Validation nodes

A validation node reads and judges, and produces nothing. It is a node of `graph.nodes` with an `id`, written with the
key `validation-node`:

```yaml
- id: check_answer
  validation-node:
    model: { id: fast, max_tokens: 1000 }
    resources: { prompt: criterion }
    message: { input: { agent_message: true } }
    on_error: stop
```

Its blocks have the same keys, in the same places, as those of a node, but only what it reads: `model`, `resources`
(only `prompt`, which is mandatory: the criterion of the judgment), `message.input`, `media.input`, `external`
(vector databases, search engines, memories) and `blackboard`. It reads the blackboard and the whole memory, both
roles, without writing them. It has no `tools`, `mcp_servers`, `skills`, `write`, `cleanup` or `output`.

Its answer has a fixed shape, `{ "validation": true | false, "reasons": "...", "log": "..." }`, which the model is
asked to follow.

- Approved: the message it examined goes on to the next node unchanged. Only the message: the media do not pass.
- Refused, with `on_error: stop`: the run ends here and `end` is not reached. The final output gets one more item, from
  the validation node, with the text `reasons: ...` and `log: ...`.
- Refused, with `on_error: node`: the run goes on, and the next node receives the problem as its `agent_message`. A
  validation node with `node` needs a node after it.

A validation node stands in the main flow only (`start`, `next`, `end`): never as the `in`, the `out` or a branch of a
group, and never inside a `react`.
