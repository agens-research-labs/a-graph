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
| `model.start` | debug | `model`, `gen_ai.request.model`, `gen_ai.provider.name`, `gen_ai.operation.name` | |
| `model.error` | error | `model`, `gen_ai.provider.name`, `error.type`, `http.response.status_code` when available, `duration`, `message` | `system_prompt`, `prompt`, `detail` |
| `model.call` | debug | `model`, `input_tokens`, `output_tokens`, `duration`, `gen_ai.*` (see below) | `system_prompt`, `prompt`, `answer`, `reasoning` |
| `tool.call` | debug | `kind` (`tool`, `mcp`, `skill`), `name`, `duration`, `is_error` | `result` |
| `structured.attempt` | debug when `valid`, warn otherwise | `model`, `mechanism` (`api`, `prompt`), `attempt`, `max_attempts`, `outcome` (`valid`, `not_json`, `incomplete`, `refused`, `schema_mismatch`), `message` | `reasons`, and `correction` when the answer is sent back |
| `utility.call` | debug | `id`, `place`, `duration` | |
| `read.vectordb` | debug | `id`, `chunks`, `duration` | |
| `read.memory` | debug | `id`, `messages`, `summary`, `duration` | |
| `read.media` | debug | `id`, `bytes`, `duration` | |
| `read.prompt` | debug | `id`, `bytes`, `duration` | |

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

Runtime records carry `trace_id` and `span_id`, shared with the exported trace. Each run gets a new trace ID,
including when a program is reused, unless the request brings a valid `traceparent` (see
[the trace of the caller](#the-trace-of-the-caller)). Before a run starts (for example during validation), these IDs
are absent. IDs are present in runtime logs even when trace export is disabled.

Model-start records include the configured `server.address` and effective `server.port`, without URL userinfo,
path or query. These remain available for cancelled calls and SDK-backed providers. Model completion records add
safe metadata observed by the shared REST transport: `server.address`, `server.port`, `url.scheme`,
`http.request.method` and `http.response.status_code` when a response was received. A network failure has no invented
HTTP status; an HTTP 200 followed by an invalid model response still records HTTP 200 alongside the model error.

These attributes describe the **first** shared-transport REST request of the logical model invocation. If that
invocation makes more than one request, `a_graph.http_request_count` records the count instead of silently replacing
the primary request's metadata with a later request's. Metadata is bounded and isolated between parallel invocations.
SDK traffic bypassing this transport gets configured destination metadata only; HTTP method/status are not guessed.
This does not add individual HTTP/retry spans or propagate `traceparent`/`tracestate`. No URL paths, query strings,
credentials, request headers or bodies are captured by this instrumentation. Reqwest network errors also omit their
URL so that query secrets cannot leak through the run's error message.

Model-call records retain `model` as the graph alias and add `gen_ai.provider.name`, `gen_ai.response.model`,
`gen_ai.operation.name` (`chat`), and `gen_ai.usage.input_tokens` / `gen_ai.usage.output_tokens`. The GenAI input
count includes cache-read and cache-write tokens; the existing `input_tokens` remains the uncached count.
`system_prompt` and `prompt` are the resolved system and current user text, not a reconstruction of the full
provider request (history, media, schemas and tool rounds are not included).

For machine-readable logs on stderr set `verbose: { level: off }`, or send the log to a separate file. Otherwise
JSON records and human progress messages share stderr. Delivery/write failure notices and the CLI's final error
summary are still plain text; use a separate log file when you need a strictly JSON-only stream on failures too.

A log that cannot be written never stops the run: the standard error says so.

## The trace

The trace is optional: without the `trace` block there is none. At the end of the run, also when it fails, the
program sends **one** POST with the whole timeline, in OTLP over HTTP with JSON, to `trace.endpoint` exactly as written
(for example `http://host:4318/v1/traces`).

- One trace for each run, with one span for the run, one for every group and every node, and inside a node one for
  every model call, tool call, MCP call, skill call, checked answer (`structured <outcome>`) and reading of a vector
  database or memory; the download of a media or a prompt from a `url` is a span inside the run, `read media <id>` or
  `read prompt <id>`. The spans have their real start and duration, so the branches of a `parallel` group overlap.
- The attributes of a span are the fields of its events, with unique keys (end values replace start values).
  Application attributes use `a_graph.*`; `gen_ai.*`, `server.*`, HTTP method/status, `url.scheme` and `error.type`
  retain their standard names.
  Contents are included only as the passport allows.
- Non-batch model invocations open a `CLIENT` span before calling the provider, and close it on success or error.
  Run, group and node spans are `INTERNAL`. Provider errors retain a bounded-category `error.type`, provider identity
  and HTTP status when the provider error exposes one; no token counts are invented for failed calls. The failing
  call's open ancestors are marked as failed too. An in-flight model call cancelled by a timeout or shutdown is
  closed at flush, marked `interrupted` with `error.type=cancelled`.
- The ids of the spans are random, so two runs in the same trace never share one.
- Other spans still open at failure are closed as `interrupted`. A failed run is marked as failed.
- The service name is `passport.adl.name`. `service.version` and the instrumentation scope version identify the
  a-graph binary (`CARGO_PKG_VERSION`); `a_graph.graph.version` identifies `passport.adl.version`.

Trace contents are disabled by default, independently of the JSON log and display. To include them for public data:

```yaml
trace:
  endpoint: http://localhost:4318/v1/traces
  capture_content: true
  max_content_bytes: 4096
```

`max_content_bytes` defaults to 4096, must be at least 1, and applies separately to each prompt, answer, reasoning or
error-detail attribute. Truncation preserves UTF-8 boundaries and adds `a_graph.content_truncated=true`. This is a
per-content-field limit, not a cap on the number of spans or the entire export. The passport can still forbid all
contents; enabling capture never bypasses redaction. Logs and the live display retain their existing passport-based
policy and are not affected by these trace-only settings.

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
  memories, and the contents, as the passport allows.

Colors are used only when the standard error is a terminal and the variable `NO_COLOR` is not set: nodes in bold cyan,
groups in bold orange with their branches drawn under them, tools in blue, errors in red, warnings in yellow,
reasoning, results and details in gray.
