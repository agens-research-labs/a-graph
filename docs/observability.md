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

Some events carry contents: the system and user prompts, the answer and the reasoning of a model, the result of a
tool, the detail of an error. They are written only when the data classification of the passport is
`sensitivity: public` with no `categories`. Otherwise every content is left out: the log writes
`"content_redacted": true`, the trace writes the attribute `a_graph.content_redacted`, and the display says that
the contents are hidden. Even for public data, traces require the separate opt-in `trace.capture_content: true`.

## The events

| Event | Level | Fields | Contents |
|---|---|---|---|
| `run.start` | info | | |
| `run.end` | info | `duration` | |
| `node.start` | info | `model` | |
| `node.end` | info | `model`, `input_tokens`, `output_tokens`, `duration`, `stop`, `verdict` for a validation node, and `corrections` for a node whose answer was checked against a schema | |
| `group.start` | info | `pattern` (`linear`, `parallel`, `concurrent`, `batch`, `react`), `branches` | |
| `group.end` | info | `pattern`, `duration` | |
| `model.start` | debug | `model`, `gen_ai.request.model`, `gen_ai.provider.name`, `gen_ai.operation.name`, and `server.address`, `server.port` when the address of the model is known | |
| `model.error` | error | `model`, `gen_ai.provider.name`, `error.type`, `duration`, `message`, `http.response.status_code` when available, and the HTTP fields except on `bedrock` (see below) | `system_prompt`, `prompt`, `detail` |
| `model.call` | debug | `model`, `input_tokens`, `output_tokens`, `duration`, `gen_ai.*`, and the HTTP fields except on `bedrock` (see below) | `system_prompt`, `prompt`, `answer`, `reasoning` |
| `tool.call` | debug | `kind` (`tool`, `mcp`, `skill`), `name`, `duration`, `is_error` | `result` |
| `structured.attempt` | debug when `valid`, warn otherwise | `model`, `mechanism` (`api`, `prompt`), `attempt`, `max_attempts`, `outcome` (`valid`, `not_json`, `incomplete`, `refused`, `schema_mismatch`), `message` | `reasons`, and `correction` when the answer is sent back |
| `utility.call` | debug | `id`, `place`, `duration` | |
| `read.vectordb` | debug | `id`, `chunks`, `duration` | |
| `read.memory` | debug | `id`, `messages`, `summary`, `duration` | |
| `read.media` | debug | `id`, `bytes`, `duration` | |
| `read.prompt` | debug | `id`, `bytes`, `duration` | |

The requests of a `batch` give no `model.start`, `model.call` or `model.error`.

`read.media` and `read.prompt` are the downloads of the media and of the prompts read from a `url`, when the run
starts (see [media](media.md#a-media-from-a-url)): `id` is the media or the prompt, `bytes` what its url answered. The
url is not written, since its query may hold a secret.

The tokens and the `duration` of `node.end` are the sum of all the model calls of the node: every round of its
tools, or every turn of a react controller. Each call is a `model.call` of its own.

Every answer that a-graph checks against a schema, of a node, a judge, a react controller or a batch, is a
`structured.attempt`: its `outcome` and a `message` that holds no content, and as `reasons` what the reader or the
validator found, which may quote the answer. An answer that is not `valid` is a warning, never an error: when it stops
the run, the error of the run follows it. `corrections` in `node.end` is how many corrections the node needed; with
the provider's API (`mechanism: api`) the answer is asked once, so it is `0`; with `schema_prompt` or
`structured_output: prompt` (`mechanism: prompt`) every attempt is a `model.call` and a `structured.attempt` of its own, the text sent back is the
content `correction` of the attempt that needed it, and the tokens of all the attempts are in `node.end`.

An error is an event too, of level `error`, with the code of its rule and the fields `phase`, `path` and `message`. A
run that fails ends with its error instead of `run.end`. A warning is an event of level `warn`.

The `message` of an error never holds a content. What made the error, written by a model, a utility, a provider or a
vector database, or held by a memory file (a verdict of a judge without `validation`, the answer of a react controller,
what a utility wrote on its standard error, the text of an error of a provider or of a vector database, an unexpected
value in the answer of a provider, a text of a memory file that is not valid) is the content `detail` of the event,
written only as the passport allows; in the trace
it is the attribute `a_graph.detail` of the interval the error ends. The caller of the run, on the command line or
through the service, gets the `message` only.

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

Every event of a run carries `trace_id` and `span_id`, the ids of the trace, also when there is no `trace` block.
Each run has a trace of its own, also when the same program runs again, unless the request brings a valid
`traceparent` (see [the trace of the caller](#the-trace-of-the-caller)). The events written before the run starts,
such as the errors of the check of the file, have no ids.

The HTTP fields of a model call:

- `model.start` has `server.address` and `server.port`: the host and the port of the model, without user, path or
  query. They are there also for a call that is cancelled, and on `bedrock`.
- `model.call` and `model.error` add what the program saw of the first HTTP request of the call: `server.address`,
  `server.port`, `url.scheme`, `http.request.method`, and `http.response.status_code` when an answer came. A network
  failure has no status; an answer 200 that cannot be read keeps its 200 next to the error. A call that made more than
  one request adds `http_request_count`, how many, and its other fields stay those of the first request. Each call
  has its own fields, also when calls run in parallel.
- On `bedrock` the calls go through the AWS SDK: `model.call` and `model.error` have none of these fields.
- No path, query, key, header or body is written. The calls to the providers get no span of their own and carry no
  `traceparent`. The message of a network error of a model call never holds the url, whose query may hold a secret.

In `model.call`, `model` is the id of the model in the graph; `gen_ai.provider.name`, `gen_ai.response.model` (the
model that answered, as the provider names it, or the model of the graph when the provider names none),
`gen_ai.operation.name` (`chat`), `gen_ai.usage.input_tokens` and `gen_ai.usage.output_tokens` follow the conventions
of OpenTelemetry. `gen_ai.usage.input_tokens` is `input_tokens` plus the tokens read from and written to the cache of
the provider. On `anthropic` and `bedrock` the provider's count of the input already leaves the cached tokens out; on
`openai`, `gemini` and `ollama` the program takes the cached tokens away from the provider's count; `mistral` gives no
count of cached tokens, so `input_tokens` is its `prompt_tokens` and the two fields are equal. `system_prompt` and `prompt` are the system text and the user text of the call:
the prompt of the node with its placeholders filled and, at the end of the system text, what a-graph adds (the
catalog of the skills, the instructions of `schema_prompt` or of `structured_output: prompt`). They are not the whole
request: the history, the media, the schemas and the rounds of the tools are left out.

With the log on the standard error and a display that is not `off`, the two are mixed. For a standard error of JSON
only, set `verbose: { level: off }`, or send the log to a file or a url. The notices of a log or a trace that cannot
be delivered, and the errors the command line writes at the end, are plain text all the same: only a log in a file
or at a url is JSON alone.

A log that cannot be written never stops the run: the standard error says so.

## The trace

The trace is optional: without the `trace` block there is none. At the end of the run, also when it fails, the
program sends **one** POST with the whole timeline, in OTLP over HTTP with JSON, to `trace.endpoint` exactly as written
(for example `http://host:4318/v1/traces`).

- One trace for each run, with one span for the run, one for every group and every node, and inside a node one for
  every model call, tool call, MCP call, skill call and checked answer (`structured <outcome>`). What a node does
  before its model is called (the utilities of its input, the reading of its vector databases, memories and media)
  and after its answer (the utilities of its output and of its blackboard, the saving of its memories and their
  summary) has its spans beside the span of the node, under the group or the run, before and after it. The download
  of a media or a prompt from a `url` is a span inside the run, `read media <id>` or `read prompt <id>`. The spans
  have their real start and duration, so the branches of a `parallel` group overlap.
- The attributes of a span are the fields of its events, with unique keys (end values replace start values). Those
  of a-graph are named `a_graph.*` (`a_graph.http_request_count`, for one); those of OpenTelemetry keep their names:
  `gen_ai.*`, `server.*`, `http.request.*`, `http.response.*`, `url.scheme`, `error.type`. Contents are included only
  as the passport allows.
- Every model call outside a batch has a span of kind `CLIENT`, opened before the call and closed when it ends, well
  or badly; the spans of the run, the groups and the nodes are `INTERNAL`. A failed call keeps `error.type`, the
  provider and the HTTP status when its error has one, and no tokens; the spans still open that hold it are marked as
  failed too. A call still going on when the run is stopped (`run_timeout`, the service shutting down, Ctrl-C on the
  command line) is closed when the trace is sent, as `interrupted`, with `error.type` `cancelled`.
- The ids of the spans are random, so two runs in the same trace never share one.
- Other spans still open at failure are closed as `interrupted`. A failed run is marked as failed.
- The service name is `passport.adl.name`. `service.version` and the version of the instrumentation scope are the
  version of a-graph; `a_graph.graph.version` is `passport.adl.version`.

The contents of the trace are off by default, whatever the log and the display do: a span that has some writes
`a_graph.content_redacted`. For public data they are turned on with:

```yaml
trace:
  endpoint: http://localhost:4318/v1/traces
  capture_content: true
  max_content_bytes: 4096
```

`max_content_bytes` (4096 by default, at least 1) is the most bytes of each content of an event (a prompt, an
answer, a result, a detail...), taken one by one: a longer one is cut at the end of a UTF-8 character, and its span
gets `a_graph.content_truncated`. It does not limit the number of spans or the size of the whole trace. A passport
that hides the contents hides them from the trace too, with or without `capture_content`; the log and the display
follow the passport alone.

### The trace of the caller

A request to the service (`POST /run`, `POST /runs`, see [serve](serve.md#running-a-graph)) may carry a
`traceparent` header ([W3C Trace Context](https://www.w3.org/TR/trace-context/)), for example
`00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`. When it is valid, the run is in that trace: the trace id
of its spans and of its log is the caller's, and the span of the run hangs from the span the header names. Valid
means: lowercase hex, a trace id and a span id that are not all zeros, a version other than `ff`, exactly the four
fields for the version `00` (a later version may add fields after a dash, which are not read), and one header only.
Anything else is as no header, never an error: the run has a trace of its own. `tracestate` is not read.

When the flag of the header is `00`, the caller does not record its trace: the run sends nothing to `trace.endpoint`,
as if there were no `trace` block, but its log still carries the trace id of the caller. Only the bit `sampled` of
the flag is read.

The answer gives back the `traceparent` of the run, so that the caller finds it: the trace, the span of the run, and
the flag `01` when the trace of the run is sent, `00` otherwise (no `trace` block, or a caller that does not record).
A request without the header gets it too, with the run's own trace.

For a runnable Collector → Jaeger demo, see [examples/03-observability](../examples/03-observability/README.md).
It verifies the OTLP trace path; JSON logs stay local and no metrics are exported.

## Delivery over the network

The log sent to a URL and the trace are delivered the same way. Each POST has a timeout of 10 seconds; a network error,
a 5xx or a 429 is tried again up to 3 times, waiting longer each time. Any other refusal is final. What cannot be
delivered is dropped, with a notice on the standard error: it never stops the run.

An OTLP HTTP 200 can report `partialSuccess`. Trace delivery reads the response (at most 64 KiB) and reports the
rejected span count, including warning-only responses with zero rejections. It never retries a partially accepted
batch. Malformed or unreadable successful responses are reported as acceptance unknown, without replaying data.
Remote error-message text is not printed. These OTLP rules do not change delivery of JSON logs.

## The display

`verbose.level` says what is shown, on the standard error, while the run goes:

- `off`: nothing;
- `steps`: the run, the nodes and the groups as they start and end, with durations and tokens, and the errors and
  warnings;
- `detail`: also the model calls, the tool calls, the answers checked against a schema
  (`structured <outcome> · <mechanism> · attempt <n>/<max>`), the utilities, the readings of vector databases and
  memories, the downloads of media and prompts from a `url`, and the contents, as the passport allows.

Colors are used only when the standard error is a terminal and the variable `NO_COLOR` is not set: nodes in bold cyan,
groups in bold orange with their branches drawn under them, tools in blue, errors in red, warnings in yellow,
reasoning, results and details in gray.
