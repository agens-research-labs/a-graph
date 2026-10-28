# The HTTP service

`a-graph --serve service.yml` starts a service that runs the graphs it is sent over HTTP: the caller sends the
text of a graph and a message, and gets the result, the same JSON as [the command line](cli.md#the-result) writes.

- The keys of the file `service.yml` are described in [the service file](service.md).
- The routes, the requests and the answers are described, in OpenAPI 3.1, in `spec/service.openapi.json`.

## Starting

```
a-graph --serve service.yml
```

The service listens on `port`, on all interfaces, and writes `a-graph: listening on port <port>` on the standard
error. A file of the service that is not valid ends the command with 1, and every error is written; a port it cannot
listen on ends it with 3.

The file holds no keys of models or services. They are environment variables of the service, and a graph names them
as always, `api_key: { env: ANTHROPIC_API_KEY }`.

The service has no authentication of its own: whoever reaches its port can run graphs on it.

## Running a graph

- `POST /run` runs the graph and answers when it is done.
- `POST /runs` starts it and answers at once with the id of the run. `GET /runs/{id}/events` then sends its events as
  they happen, as Server-Sent Events, and `GET /runs/{id}` tells its state. A run is kept for `retention_seconds`
  after it ends, then it is forgotten. Its id is random: whoever knows it can watch the run.
- `GET /health` tells that the service is up.

At most `max_runs` runs go on at once (8 when the file does not say), those of `POST /run` and of `POST /runs`
together. A request beyond it is checked as always, then refused with 503 (`too_many_runs`): it is not queued, the
caller tries again later.

The graph is checked and built again for each request: requests share nothing but the folders of their sessions. A
request weighs at most 64 MiB.

## Sessions

A request may name a `session`, a name of letters, digits, `-`, `_` and `.`, at most 64 characters; without it, the
session is `default`. Each session has its own folder, `<data_dir>/<session>/`, created at its first request. Two
requests with the same session use the same folder, so a memory or a blackboard file written by one is read by the
next.

## A graph from outside

A graph sent to the service comes from outside, so it is not trusted. Before anything is built, it is refused (422)
if it breaks one of these rules:

- **No programs.** A tool, an MCP server or a utility with `transport: stdio` is refused, and so is a skill without
  `run_scripts: false`: only services reached over `http` are allowed (`not_supported`).
- **Files in the session.** Every path of the graph (the `path` of a prompt, a media, a skill or a memory, the `file`
  of a blackboard or of the log) is moved into the folder of the session. It must be relative and without `..`
  (`not_supported`). A file the graph reads must therefore already be in the folder of the session; a media can come
  inside the graph instead, as `source: { base64 }`.
- **Hosts.** Every host in the `allowed_hosts` of the passport of the graph must be in the `allowed_hosts` of the
  service (`network_not_allowed`): a graph can only restrict the hosts of the service. An entry of the graph,
  `host` or `host:port`, is accepted when the service lists the same entry or the host alone. An entry `host` is
  refused when the service lists the host only with a port, because the graph would allow every port.

Then the graph is checked as by `--check`: a graph that is not valid is refused (422) with all its errors, in the form
`{ "errors": [ { "phase", "path", "rule", "message" } ] }`.
