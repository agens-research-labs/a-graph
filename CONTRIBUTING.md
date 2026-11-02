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
`pymupdf4llm`, or the keys `GEMINI_API_KEY` and `CLAUDE_API_KEY` for the real APIs.

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
docker build .                  # the image, once the Dockerfile is in the repository
```

The same commands run by hand, before opening a pull request. Run `cargo fmt` first: CI checks the formatting, it does
not fix it. Clippy fails on any warning: a lint that does not fit the code is allowed in `[lints.clippy]` of
`Cargo.toml`, with the reason next to it. A new release of Rust brings new lints: the bump of `rust-toolchain.toml` is
a commit of its own, with what the new lints asked for. `cargo audit` fails on a new advisory: the fix is a
`cargo update` of that crate, or, when there is no fix yet, an entry in `.cargo/audit.toml` with the reason.

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
- `src/service`: the HTTP service.
- `src/error`: the errors, with their phase, path and rule.

A file is read in two steps. First the loader checks its shape and the links between its names (a name is unique in
the whole file). Then each block is built by its own manager, which checks the rules of its content. The errors of
all the steps are reported together.

## Tests

- Every test lives in `tests/<topic>/<name>/`, with its code (`<name>.rs`), its graph file (`<name>.yml`) and its
  files (`files/`). Each topic has a `tests/<topic>/main.rs` that lists its tests.
- The names of the tests are short.
- Code shared by the tests lives only in `tests/common`, and the MCP server used by the tests in `tests/mcp_server`, a
  project of its own.
- The mock servers of the tests listen on fixed ports, written in each test's yml, so that no port is shared between
  tests.
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
