# A-Graph

`a-graph` runs a **graph of agents written in a YAML file**. You describe the models, the data they read, the tools
they use and how the nodes are connected; `a-graph` checks the file, runs it, and gives you the result. It is a program
for the command line and a small HTTP service, written in Rust.

```
a-graph graph.yml "What is a mutex?"
```

Inspired by [KeGal](https://github.com/Kedos-srl/kegal), this Rust implementation was designed with a configuration-first approach, with the code following
the configuration design.

## Build

You need [Rust](https://rustup.rs). From the root of this project:

```
cargo build --release
```

The program is `target/release/a-graph`, about 53 MB (about 35 MB without its symbols, after `strip`). To build it and
put `a-graph` in `~/.cargo/bin`, which `rustup` already adds to the `PATH`:

```
cargo install --path . --locked
```

The code has been built and tested on Linux and on macOS: the CI runs the tests on Linux for every pull request,
and on macOS too after every push to `main`.

The builds of development and the tests are kept in `target/`, which takes about 9 GB after the whole suite and grows
with every build, since Cargo does not remove the old ones: `cargo clean` empties it. To keep it smaller, `Cargo.toml`
leaves out the debug information of the dependencies in the builds of development
(`[profile.dev.package."*"] debug = false`); the code of the project keeps it whole.

## A first run

The examples need no key: they use a local [Ollama](https://ollama.com) with `qwen3.5:4b`.

```
ollama pull qwen3.5:4b
cd examples/01-hello
a-graph graph.yml "What is a mutex?"
```

The progress of the run is written on the standard error; the result is a JSON document on the standard output:

```json
{"items":[{"node":"assistant","message":"A mutex is a lock that lets only one thread use a shared resource at a time.","media":[],"reasoning":[],"logprobs":[]}]}
```

`examples/02-sequence` connects two nodes: the first translates the message, the second shortens the translation. Run
each example from its own folder. [`examples/03-observability`](examples/03-observability/README.md) adds correlated
JSON logs and traces through a Docker Compose OpenTelemetry Collector and an in-memory Jaeger viewer.

## The command line

```
a-graph graph.yml "the message"             # checks the file, then runs it
cat question.txt | a-graph graph.yml        # without a message, the standard input is the message
a-graph graph.yml "the message" --chat_mode # only `node: text`, one line for each result
a-graph --check graph.yml                   # only checks the file: nothing runs
a-graph --serve service.yml                 # starts the HTTP service
```

The keys of the models and services are environment variables: the graph writes their names
(`api_key: { env: ANTHROPIC_API_KEY }`), never their values. Everything about the command line, the result, the errors
and the exit codes is in [the command line](docs/cli.md).

## The graph file

A graph is one YAML file, made of blocks:

| Block | What it holds |
|---|---|
| `graph_spec`, `passport` | the version of the format and the identity of the graph ([ADL 0.3.0](https://www.adl-spec.org/)): name, data classification, **the hosts the graph may reach** |
| `observability`, `trace`, `verbose` | the log, the timeline sent to a collector, the live display of the run |
| `models` | the language models |
| `embeddings` | the models that turn a text into a vector |
| `utilities` | programs and services of yours, called by `a-graph` on the data of a node |
| `resources` | `prompts`, `tools`, `mcp_servers`, `media`, `skills` |
| `external` | `vectordb`, `search`, `memory` |
| `blackboards` | Markdown notes shared by nodes |
| `graph` | the `nodes` and the `edges` that connect them |

How the file works is explained in [the graph file](docs/file.md). Every key is described in
[the reference](docs/graph.md), and the JSON Schema of the file is [`spec/graph.schema.json`](spec/graph.schema.json):
both are generated from the code.

## What it does

- **Models**: Anthropic, OpenAI (Responses API), Gemini, Mistral, Amazon Bedrock, Google Vertex AI, Microsoft Foundry,
  Ollama, and any server that speaks the Ollama contract. Reasoning levels, output schemas (JSON), batch jobs, images
  in, images and audio out.
- **Edges**: nodes in sequence; groups whose branches run one after the other, all together, at most a few at a time, or
  as one batch job; a `react`, where a controller chooses at every turn which worker to call; validation nodes that
  judge a result and can stop the run.
- **Tools**: your own programs (`stdio`) or services (`http`) with a small JSON contract; **MCP servers** (stdio and
  Streamable HTTP); **skills** in the [Agent Skills](https://agentskills.io/specification) format.
- **Utilities**: programs in any language that change the data of a node before or after the model, without the model
  knowing: convert a PDF into Markdown, hide personal data, add a mark.
- **Data**: vector databases (Qdrant, Chroma) searched with an embedding model, whose text goes into the prompt as
  `{{chunks}}`; a conversation **memory** in a JSON file, with summaries and, if wanted, a search by similarity;
  **blackboards** shared by nodes.
- **Observability**: a JSON log (standard error, file or URL), a **trace** sent as OpenTelemetry (OTLP over HTTP, JSON) at
  the end of the run, and a live display. The contents of the messages appear in them only if the data classification
  of the passport allows it.

## What it does not do yet

- A blackboard kept at a `url`: the file is refused when it is checked.
- `search`, and a memory kept in a vector database (`provider: vectordb`): the file is accepted, but a node that uses
  one fails when it runs.
- Audio and video in: no provider takes them yet.
- A `react` inside a `react`.
- The batch of Gemini, and of Vertex AI with publisher `google`.

## The HTTP service

`a-graph --serve service.yml` starts an HTTP service: a caller sends the text of a graph and a message, and gets the
same JSON as the command line. Since the graph comes from outside, the service refuses, before it builds anything,
what could start a program, touch a file outside the folder of the session, or reach a host it does not allow; and
it refuses a graph that names an environment variable it does not list, or sends one where it may not go. It has no
authentication of its own. See [the HTTP service](docs/serve.md), which also says how to run it with Docker
(`docker compose up -d`); the routes are described, in OpenAPI 3.1, in [`spec/service.openapi.json`](spec/service.openapi.json).

## Documentation

All the documentation is in [`docs/`](docs/README.md). To work on the code, see [CONTRIBUTING](CONTRIBUTING.md).

## License

[MIT](LICENSE).
