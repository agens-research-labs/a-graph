# Observability

A run reports on itself in three separate ways, each with its own block:

- `observability`: the **log**, structured records of what the program does, for machines;
- `trace`: the **timeline** of the run, sent to a collector at the end, for later analysis;
- `verbose`: the **display**, the progress of the run as it goes, for a person.

All three read the same events. Every key is described in [the reference](graph.md).

```yaml
observability: { level: info }
trace: { endpoint: http://localhost:4318/v1/traces }
verbose: { level: steps }
```

## Contents and the passport

Some events carry contents: the prompt, the answer and the reasoning of a model, the result of a tool. They are written
only when the data classification of the passport is `sensitivity: public` with no `categories`. Otherwise every
content is left out: the log writes `"content_redacted": true`, the trace writes the attribute
`a_graph.content_redacted`, and the display says that the contents are hidden.

## The events

| Event | Level | Fields | Contents |
|---|---|---|---|
| `run.start` | info | | |
| `run.end` | info | `duration` | |
| `node.start` | info | `model` | |
| `node.end` | info | `model`, `input_tokens`, `output_tokens`, `duration`, `stop`, and `verdict` for a validation node | |
| `group.start` | info | `pattern` (`linear`, `parallel`, `concurrent`, `batch`, `react`), `branches` | |
| `group.end` | info | `pattern`, `duration` | |
| `model.call` | debug | `model`, `input_tokens`, `output_tokens`, `duration` | `prompt`, `answer`, `reasoning` |
| `tool.call` | debug | `kind` (`tool`, `mcp`, `skill`), `name`, `duration`, `is_error` | `result` |
| `utility.call` | debug | `id`, `place`, `duration` | |
| `read.vectordb` | debug | `id`, `chunks`, `duration` | |
| `read.memory` | debug | `id`, `messages`, `summary`, `duration` | |

The tokens and the `duration` of `node.end` are the sum of all the model calls of the node: every round of its
tools, or every turn of a react controller. Each call is a `model.call` of its own.

An error is an event too, of level `error`, with the code of its rule and the fields `phase`, `path` and `message`. A
run that fails ends with its error instead of `run.end`. A warning is an event of level `warn`.

## The log

`observability.level` is the lowest level written: `info` writes the run, the nodes and the groups, `debug` also the
calls, and `warn` and `error` only the problems.

Each event is one JSON object on one line, with its fields and the keys `ts` (UTC, RFC 3339), `level`, `code` and,
when there is one, `node`.

`observability.output` says where the log goes:

- `stderr` (the default): with the errors, so that the standard output holds only the result;
- `stdout`: mixed with the result;
- `{ file: <path> }`: a file;
- `{ url: <endpoint>, headers: ... }`: an HTTP endpoint, as `application/x-ndjson` POSTs of at most 100 events each.

A log that cannot be written never stops the run: the standard error says so.

## The trace

The trace is optional: without the `trace` block there is none. At the end of the run, also when it fails, the
program sends **one** POST with the whole timeline, in OTLP over HTTP with JSON, to `trace.endpoint` exactly as written
(for example `http://host:4318/v1/traces`).

- One trace for each run, with one span for the run, one for every group and every node, and inside a node one for
  every model call, tool call, MCP call, skill call and reading of a vector database or memory. The spans have their
  real start and duration, so the branches of a `parallel` group overlap.
- The attributes of a span are the fields of its events; contents only as the passport allows.
- The span of a failed run is marked as failed. The spans still open when the run failed are closed as
  `interrupted`.
- The service name is `passport.adl.name`.

## Delivery over the network

The log sent to a URL and the trace are delivered the same way. Each POST has a timeout of 10 seconds; a network error,
a 5xx or a 429 is tried again up to 3 times, waiting longer each time. Any other refusal is final. What cannot be
delivered is dropped, with a notice on the standard error: it never stops the run.

## The display

`verbose.level` says what is shown, on the standard error, while the run goes:

- `off`: nothing;
- `steps`: the run, the nodes and the groups as they start and end, with durations and tokens, and the errors and
  warnings;
- `detail`: also the model calls, the tool calls, the utilities, the readings of vector databases and memories, and
  the contents, as the passport allows.

Colors are used only when the standard error is a terminal and the variable `NO_COLOR` is not set: nodes in bold cyan,
groups in bold orange with their branches drawn under them, tools in blue, errors in red, warnings in yellow,
reasoning, results and details in gray.
