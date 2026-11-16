# Correlated logs and traces

The hello graph with a local OpenTelemetry Collector and a Jaeger trace viewer:

```text
a-graph (host) -- OTLP/HTTP JSON :4318 --> Collector -- OTLP/gRPC --> Jaeger
                                                                    |
                                                        http://localhost:16686
```

Docker with Compose, Rust, and Ollama with `qwen3.5:4b` are required. From the repository root:

```sh
cargo build --locked
ollama pull qwen3.5:4b
docker compose -f examples/03-observability/compose.yml up -d
mkdir -p target/o11y-real
./target/debug/a-graph examples/03-observability/graph.yml "What is a mutex?" \
  > target/o11y-real/result.json 2> target/o11y-real/events.ndjson
```

Wait for the Collector to report `Everything is ready` in `docker compose -f
examples/03-observability/compose.yml logs collector` before running the graph.

Open <http://localhost:16686>, select service **hello-observability**, and find traces. Allow a few
seconds for the Collector's batch to arrive. Each run has three spans: `run` → `node assistant` →
`model local`. Expand the model span to inspect provider, actual model, tokens, system prompt,
user prompt and answer.

The JSON log's `trace_id` is also a direct lookup:

```sh
# With jq installed:
TRACE_ID=$(jq -r 'select(.code == "run.start") | .trace_id' target/o11y-real/events.ndjson)
curl --fail "http://localhost:16686/api/traces/$TRACE_ID"
# Browser: http://localhost:16686/trace/<trace_id>
```

`verbose: off` keeps stderr machine-readable on a successful run; stdout is only the result.
Delivery failures can still produce plain-text notices on stderr. Check the log for delivery errors
if the trace is absent; a successful model call alone does not prove trace delivery.

## Scope and safety

- This is a **trace** pipeline. Application JSON logs remain local; they are not OTLP logs and must
  not be sent directly to the Collector's OTLP logs endpoint. There is no metrics pipeline here.
- The passport deliberately declares **public** data and the demo sets `trace.capture_content: true`:
  system/user prompts and answers are exported, capped at 4096 UTF-8 bytes per field. The default outside
  this demo is no trace content. Use non-public classification for sensitive inputs; contents will be redacted.
- Model calls are CLIENT spans with requested/returned model metadata. Failures retain their own error span;
  in-flight calls cancelled at shutdown/timeout are retained as interrupted. Resource attributes identify
  both the a-graph binary version and graph version.
- Only Collector ingestion (`4318`) and the Jaeger UI (`16686`) are exposed, on loopback. No auth or
  TLS is configured. Container-internal OTLP is plaintext. This is a local demo, not a production
  security configuration.
- Jaeger uses memory, capped at 1,000 traces. Restarting it loses all traces. Container memory limits
  are 256 MiB for Collector and 512 MiB for Jaeger.
- Images are version-pinned. Jaeger v1 is a demo-only legacy backend; a production deployment should
  use a maintained backend, persistent storage, authentication and TLS. The app continues to send
  the same OTLP payload to a Collector when the backend changes.

Stop the demo with:

```sh
docker compose -f examples/03-observability/compose.yml down
```
