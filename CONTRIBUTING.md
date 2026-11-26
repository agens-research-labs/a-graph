# Contributing

## Build and test

You need [rustup](https://rustup.rs): the Rust of the project is the one of `rust-toolchain.toml`, and rustup
installs it by itself at the first `cargo`. From the root of the project:

```
cargo build
cargo test                      # the whole suite: no network and no key needed
cargo test --test memory        # the tests of one topic
cargo test -- --ignored         # the live tests (see below)
```

The suite runs against mock servers, started by the tests themselves: it needs no network and no key.

The live tests are marked `#[ignore]` because they need something outside the project; each says what in its
`#[ignore]` line. They need a local [Ollama](https://ollama.com) with some models (`qwen3.5:4b`, `embeddinggemma`,
`llama3.2`, `gpt-oss`, `mistral-small3.2`, `gemma4:e2b`), a Chroma server on `127.0.0.1:8000`, `python3` with
`pymupdf4llm`, or the keys of the real APIs: `GEMINI_API_KEY`, `CLAUDE_API_KEY`, `OPENAI_API_KEY`, `MISTRAL_API_KEY`,
and `AWS_ACCESS_KEY_ID` with `AWS_SECRET_ACCESS_KEY` for Bedrock.

The code has been built and tested on Linux and on macOS. On macOS, in a parallel run, a test that starts a
program (`utilities`, `tools`) can fail at random with "did not answer within 5 s": the standard library creates the
pipes of a child in two steps there, and a child started by another test in between can inherit one and hold it
open. It does not happen on Linux, where the pipes are created closed on exec. Run the test again, alone.

## The checks

Every pull request and every push to `main` run the workflow `.github/workflows/ci.yml` on GitHub, on Linux, with
the Rust of `rust-toolchain.toml`. A push to `main` runs the tests on macOS too, which a pull request does not: run
them by hand on the other system when the change touches files, processes or timing.

```
cargo fmt --check
cargo build --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked --no-fail-fast
cargo audit                     # the advisories on the crates of Cargo.lock (cargo install cargo-audit --locked)
docker build .                  # the image of the service
```

The same commands run by hand, before opening a pull request. CI checks the formatting, it does not fix it: let the
editor run rustfmt on save, and `cargo fmt` formats everything by hand. The width is in `rustfmt.toml` and the version
of rustfmt in `rust-toolchain.toml`, so every editor formats the same way.

- VS Code, Cursor: with the rust-analyzer extension, `"[rust]": { "editor.formatOnSave": true }` in the settings.
- RustRover: Settings, Rust, Rustfmt, "Run rustfmt on Save".
- Zed: on by default.
- A coding agent: `cargo fmt` before every commit.

Clippy fails on any warning: a lint that does not fit the code is allowed in `[lints.clippy]` of
`Cargo.toml`, with the reason next to it. A new release of Rust brings new lints: the bump of `rust-toolchain.toml` is
a commit of its own, with what the new lints asked for. `cargo audit` fails on a new advisory: the fix is a
`cargo update` of that crate, or, when there is no fix yet, an entry in `.cargo/audit.toml` with the reason.

## Commits and branches

The rules for branches and commits are in [AGENTS.md](AGENTS.md), and hold for people as for coding agents. The main
ones:

- no commit on `main`: the work goes on a branch named `type/short-kebab-description`;
- the title of a commit is `type(scope): description`, with the types and the scopes listed there;
- one change for each commit, with the documentation it needs in the same commit.

The hook in `.githooks/commit-msg` checks the message of every commit against those rules, and refuses the commit
when one is broken. Turn it on once, in your clone:

```
git config core.hooksPath .githooks
```

CI runs the same hook on every commit of a pull request, so a commit made without it is caught there. The hook
cannot check the imperative mood or whether a commit holds one change: that is left to the review. A
`git commit --fixup` passes on a branch and fails in CI, so squash it before asking for the merge.

## How the code is organized

- `src/main.rs`: the command line.
- `src/schema`: the names of the keys and the shape of each block of the graph file.
- `src/loader`: checks the shape of the file, the links between ids and the order of the edges.
- `src/models`, `src/embeddings`, `src/resources`, `src/external`, `src/blackboards`, `src/utilities`: the blocks of
  the file, each built by its own manager.
- `src/graph`: the nodes and the edges, and the rules between them.
- `src/run`: the execution of a graph.
- `src/gate`: the calls to tools, MCP servers and skill scripts.
- `src/logging`: the log, the trace and the display.
- `src/network.rs`: the check of the allowed hosts.
- `src/secrets.rs`: the check of the environment variables a graph sent to the service may use.
- `src/service`: the HTTP service.
- `src/error`: the errors, with their phase, path and rule.

A file is read in two steps. First the loader checks its shape and the links between its names (a name is unique in
the whole file). Then each block is built by its own manager, which checks the rules of its content. The errors of
all the steps are reported together.

## Adding a provider

A provider of the yml (`provider: ...`) is a value of `Model`; the code that calls it is an implementation of the
trait `Llm` (`src/models/llm.rs`). There is one implementation for each API format, not for each company: `custom` is
the code of `ollama` with another name, `azure_foundry` the code of `openai` or `anthropic` with another host and
other credentials, `vertex_ai` the code of `anthropic` or `gemini` with Google credentials.

First decide which case it is.

**The API of an existing provider, at another host or with other credentials.** No new folder: a constructor next
to the others (`OpenAi::azure`, `Anthropic::vertex`, `Ollama::custom`) that sets the base URL, the authentication
and the name of the provider in the errors.

**A new API format.** A folder `src/models/<name>/`, like the others:

- `mod.rs`: the struct, its constructor, its `ERRORS` and `impl Llm`;
- `request.rs`: a `ModelInvocation` written as the body the API expects;
- `response.rs`: the answer read as a `ModelResponse`;
- `batch.rs`, only if the API has batch jobs.

What `impl Llm` must keep:

- `check` refuses, without any call, what the API cannot do (`ModelRequirements`: reasoning, schema, media, tools,
  batch), all the problems in one `ModelError::Unsupported`. A provider without batch refuses `req.batch` here and
  answers `invoke_batch` with `unsupported`.
- `invoke` makes one call, one round: the loop of the tools belongs to the node. The `StopReason` says why the answer
  ended; `MaxTokens` and `ContextExceeded` are cut answers: one that asks for a tool is `Identity::truncated`, and its
  tool calls are never run. With a schema the text goes back as it came, cut or not: the JSON is read by
  `models::structured`. A refused answer is `Refused`, with no tool calls.
- The calls go through `http::send` (or `send_bytes`) with the `ErrorShape` of the provider: the JSON pointers of the
  code and of the message of its error answers, and which answers are about the credentials. `api_error` reads them:
  the message of the error says the status and the code, what the provider wrote is its `said`, written only where
  the passport allows contents. Every other error is built with the methods of `Identity`. `bedrock` is the
  exception: it calls through the AWS SDK and maps the errors of the SDK itself.
- `cancel_batch` stops a job that the run let go of; its default does nothing.

In both cases:

- `src/schema/models.rs`: the `Variant` of the provider, its fields and their descriptions; then the five generated
  files (see [The schema of the graph file](#the-schema-of-the-graph-file)).
- `src/models/spec.rs`: the value of `Provider`, the variant of `Model` and its reading in `read`.
- `src/models/endpoint.rs`: the host of the model, its destinations and its secrets, for `allowed_hosts` and
  `allowed_env`.
- `src/models/manager.rs`: the variant in `build_llm`.
- `docs/models.md`: what the provider supports and the code of its errors.
- `tests/models/<name>/`: the calls against a mock server, at least a plain call, the stop reasons and the error
  answers (`error_messages`).

## Tests

- Every test lives in `tests/<topic>/<name>/`, with its code (`<name>.rs`), its graph file (`<name>.yml`) and its
  files (`files/`). Each topic has a `tests/<topic>/main.rs` that lists its tests.
- The names of the tests are short.
- Code shared by the tests lives only in `tests/common`, and the MCP server used by the tests in `tests/mcp_server`, a
  project of its own.
- The mock servers of the tests listen on a free port, chosen when the test starts: the yml names it
  `{{port:<name>}}` and the test fills it in, so that no port is shared between tests, nor between two `cargo test`
  at once.
- A test must run from a clean checkout, for anyone: no file outside the project, no state left by another test.

## The schema of the graph file

The keys of the graph file and of the file of the service, their shapes and their descriptions are written once, in
`src/schema`. From there the program generates the JSON Schemas `spec/graph.schema.json` and
`spec/service.schema.json` and the references `docs/graph.md` and `docs/service.md`, which are never edited by hand.
The routes of the HTTP service are listed once, in `src/service/api.rs`: the server is built from that list, and the
OpenAPI description `spec/service.openapi.json` is generated from it. After a change in `src/schema`, write them again:

```
cargo run -- --print-schema > spec/graph.schema.json
cargo run -- --print-docs > docs/graph.md
cargo run -- --print-service-schema > spec/service.schema.json
cargo run -- --print-service-docs > docs/service.md
cargo run -- --print-service-api > spec/service.openapi.json
```

The tests of `tests/schema` fail when these files are not those of the code, or when a key has no description.

## License

By contributing you agree that your contribution is released under the [MIT License](LICENSE).
