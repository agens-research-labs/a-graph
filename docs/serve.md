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
as always, `api_key: { env: ANTHROPIC_API_KEY }`. The file says which variables a graph may name and where each may
be sent, in `allowed_env`:

```yaml
allowed_env:
  ANTHROPIC_API_KEY: [api.anthropic.com]
```

A graph may then use `ANTHROPIC_API_KEY` for a model `anthropic`, but not in a header to another host, and no
other variable at all. The rules are in [Variables](#a-graph-from-outside), below.

The service has no authentication of its own: whoever reaches its port can run graphs on it.

## Stopping

`SIGTERM` (what `docker stop` sends) or `SIGINT` (Ctrl-C) stops the service gently. It writes `a-graph: shutting
down`, takes no new request, and waits for the requests and the runs going on to end, those of `POST /runs` too.
It waits at most 30 seconds. Then the runs still going on are stopped: each ends as failed, with the rule
`service_stopped`, and still ends as any run does (its batch jobs still going on are cancelled at the provider, its
last event is logged and its trace is sent). The service waits at most 5 more seconds for that, with the connections
open: a caller still waiting on `POST /run` gets its 502 with `service_stopped`, a watcher of `GET /runs/{id}/events`
gets the `error` event. Then it exits, one second later at most: work still going on in the background (a request
being prepared, a write of a blackboard waiting for its lock) is left behind. A run that did not close in time is cut,
with the connections still open: the service writes how many, `a-graph: N runs cut`. Either way it ends with 0.

Docker kills a container 10 seconds after `docker stop` by default. To give the runs their 30 seconds, and 5 more
to close, `compose.yml` sets `stop_grace_period` higher; set it in your own compose file too:

```yaml
services:
  a-graph:
    stop_grace_period: 40s
```

## Running a graph

- `POST /run` runs the graph and answers when it is done.
- `POST /runs` starts it and answers at once with the id of the run. `GET /runs/{id}/events` then sends its events as
  they happen, as Server-Sent Events, and `GET /runs/{id}` tells its state. A run is kept for `retention_seconds`
  after it ends, then it is forgotten. Its id is random: whoever knows it can watch the run.
- `GET /health` tells that the service is up.

At most `max_runs` runs go on at once (8 when the file does not say), those of `POST /run` and of `POST /runs`
together. A request beyond it is refused with 503 (`too_many_runs`) before its body is read, so it costs nothing: it
is not queued, the caller tries again later. A caller still sending the body when it is refused gets the 503, then
the connection is closed, so its sending may fail. A request holds its place from the start, while its body arrives
(at most `read_timeout_seconds`) and until its run ends. A run of `POST /run` whose caller hangs up goes on to its
end, and holds its place until then.

A run lasts at most `run_timeout_seconds` (an hour when the file does not say). Then it is stopped and ends as
failed, with the rule `run_timeout`: `POST /run` answers 502, and a run of `POST /runs` ends with the state `failed`
and the event `error`. It still ends as any run does: its MCP servers are closed, its batch jobs still going on are
cancelled at the provider (see [models](models.md#batch)), its last event is logged and its trace is sent.

A run keeps its last 1000 events. A watcher that comes after older ones were dropped first gets the event `dropped`,
`{ "count": n }`, with how many it missed, then the events kept.

The cap is on the number of events, not on their size. When the `verbose` of the graph is `detail` and its passport
allows the contents, an event can carry a long text, a prompt or an answer. The events and the outcome of a run stay
in memory until it is forgotten, `retention_seconds` after its end, and a run that ended no longer counts towards
`max_runs`. So the memory held is that of every run of the last `retention_seconds`, not only of the `max_runs`
going on. The logprobs of a node weigh on that memory too: with `logprobs: 20` every token of the answer carries 21
entries, each with its bytes, so an answer of 4000 tokens takes some MB in the result.

The graph is checked and built again for each request: requests share nothing but the folders of their sessions. A
request weighs at most 64 MiB, and its body must arrive within `read_timeout_seconds` (60 seconds when the file does
not say): a request still arriving then is refused with 408 (`read_timeout`), and its connection is closed.

At most `max_reading_per_address` requests of the same address may have their body still arriving at once (2 when the
file does not say), for `POST /run` and `POST /runs` together; a request of that address beyond it is refused with
503 (`too_many_runs`) before its body is read, even while `max_runs` still has a free place. Its place is given back
as soon as the body is read or refused. A reverse proxy that buffers the whole body before passing the request on, as
nginx does by default (`proxy_request_buffering on`), never sends a request whose body is still arriving, so it never
reaches this cap; every request then comes from the proxy's own address.

While a request is checked and built, its graph is held in memory about three times over: a request of 64 MiB, such
as a graph with a large media in `base64`, takes about 200 MB until it is built. Up to `max_runs` requests are checked
at once, so with the default of 8 the service may need about 1.2 GB for them alone. A run also keeps what the urls
of its media and prompts answered, at most 64 MiB for all of them together (see
[media](media.md#a-media-from-a-url)), until it ends: 512 MiB more for 8 runs. Choose `max_runs` for the memory the
service has.

## Sessions

A request may name a `session`, a name of letters, digits, `-`, `_` and `.`, at most 64 characters and not made of
dots only; without it, the session is `default`. Each session has its own folder, `<data_dir>/<session>/`,
created at its first request. A name
keeps its case: `A` and `a` are two sessions with two folders, unless the filesystem of `data_dir` ignores case (as
on macOS), where they are one folder. Two
requests with the same session use the same folder, so a memory or a blackboard file written by one is read by the
next.

The `default` session is shared by every caller that names none. Give each caller its own session if their files must
not meet.

Nothing deletes a session by itself, unless the service file says so.

- `session_retention_seconds` deletes the folder of a session (the whole folder, with its memories, blackboards and
  media) when nothing touched it for that long. Every request accepted touches its session, and so does the end of its
  run: it sets the modified time of the file `.a-graph-session` of the folder, which needs only the right to write it
  (a failure is written on the standard error). A refused request does not touch it. A session with a run going on is
  never deleted. Without this key nothing is ever deleted.
- `session_max_bytes` is the most a session folder may weigh. A request for a session over the cap is refused (422,
  `session_full`) before it runs. The refusal does not touch the session, so a full session is still deleted after
  `session_retention_seconds`, however often it is asked for. Use another session, or ask the operator of the service
  to free it. Without this key there is no cap.

How the sweep works:

- The folders are looked at every 10 seconds, or every `session_retention_seconds` when that is shorter. A session
  therefore lives up to about its retention plus that interval.
- Only the folders the service made are ever deleted: those with a valid session name and the empty file
  `.a-graph-session`, which every request writes in its folder. Keep `data_dir` for the service alone all the same.
- A folder made by an older version of the service has no `.a-graph-session`, so it is never deleted, unless a
  request on it writes the file, or the operator creates the empty file `.a-graph-session` in it.
- The time the service is stopped counts as idle time. After a long stop, the first look deletes every session not
  touched for longer than the retention.
- A folder to delete is first renamed `<session>~deleting` (a name no session can have), then removed, so a request
  for the same session starts a new folder. Its file `.a-graph-session` is removed last: a folder `*~deleting` left by
  a stop still has it, and is removed at the next look. A folder `*~deleting` without `.a-graph-session` is not one
  the service set aside: it stays, unless it is empty. A folder that cannot be removed is written on the standard
  error, and the service goes on.

What the cap does not do:

- The size is checked once the graph is checked, before the run. Every request already accepted may go past the cap,
  and a single run may grow with no limit (a loop writing a blackboard, media): only the next request is refused.
- The cap is per session. The service has no authentication, so the number of sessions is not limited: the disk used
  in all is bounded only by the retention and by the rate of the requests.
- The size is the sum of the lengths of the files. Blocks on the disk, folders and links are not counted.
- When the cap is set, every request walks the whole folder of its session. A session with very many files costs
  that walk at every request.

## A graph from outside

A graph sent to the service comes from outside, so it is not trusted. Before anything is built, it is refused (422)
if it breaks one of these rules:

- **No aliases.** A YAML alias (`*name`, which repeats the value marked `&name`) is refused (`not_supported`):
  every alias would copy its value, so a small graph could fill the memory of the service. Write the value again
  where it is used. A `*` or a `&` inside a text is not an alias and is allowed. The command line still accepts
  aliases.
- **At most 100000 YAML events.** Every value, key, list and map is an event (a long text, such as a media in
  `base64`, is one). A graph with more is refused (`not_supported`), before it is read whole: each event costs
  memory while the graph is checked, so a body of 64 MiB with tens of millions of small values could take
  gigabytes. Real graphs have far fewer: the largest of this repository has under 800. The command line has no such
  limit.
- **No tags.** A YAML tag (`!name` before a value, as in `path: !x notes.md`) is refused (`not_supported`): the
  rules below would not see the value under the tag, while the graph would still use it. Write the value without
  its tag. A `!` inside a text is not a tag and is allowed. The command line still accepts tags.
- **No programs.** A tool, an MCP server or a utility with `transport: stdio` is refused, and so is a skill without
  `run_scripts: false`: only services reached over `http` are allowed (`not_supported`).
- **Files in the session.** Every path of the graph (the `path` of a prompt, a media, a skill or a memory, the `file`
  of a blackboard or of the log) is moved into the folder of the session. It must be relative and without `..`
  (`not_supported`), and it cannot be `.a-graph-session`, the file the service keeps at the top of the folder, in
  any case (`not_supported`): the same name in a subfolder is allowed. A file the graph reads must therefore already be in the folder of the session; a media can come
  inside the graph instead, as `source: { base64 }`, and a media or a prompt can come from a host of
  `allowed_hosts`, as `source: { url }`.
- **Hosts.** Every host in the `allowed_hosts` of the passport of the graph must be in the `allowed_hosts` of the
  service (`network_not_allowed`): a graph can only restrict the hosts of the service. An entry of the graph,
  `host` or `host:port`, is accepted when the service lists the same entry or the host alone. An entry `host` is
  refused when the service lists the host only with a port, because the graph would allow every port.
- **Credentials of the service.** A model with `credentials: ambient` runs with the credentials of the service
  itself, found by the SDK of the provider: its AWS role or variables, its Google Application Default Credentials,
  its Azure managed identity or CLI login. It is refused unless its provider is in the `allowed_ambient` of the
  service (`ambient_not_allowed`). A `vertex_ai` model is always `ambient`. With `azure_foundry` the service may start
  the programs `az` and `azd` to get a token.

Then the graph is checked as by `--check`: a graph that is not valid is refused (422) with all its errors, in the form
`{ "errors": [ { "phase", "path", "rule", "message" } ] }`. One more rule is checked there, with the hosts of the
graph:

- **Variables.** Every `{ env: VAR }` of the graph must name a variable of `allowed_env`, and the value must go only
  to hosts listed for that variable (`env_not_allowed`): the host of the model for a key of a model, the `url` of a
  tool, a utility, an MCP server, a vector database, the log, a media or a prompt for their headers or key, the
  `endpoint` of the
  trace for its headers. A refused variable is not read, so the answer never tells whether it is set; a variable of
  the list that is not set is `env_missing`, as on the command line. The keys of a `bedrock` model go to every host
  of the model: with a `batch`, also the control API and the S3 buckets. The variables of the log are checked with
  its `url`, before anything else: a log that breaks the rule is refused with that error alone.

Last, if the service file sets `session_max_bytes`, a session whose folder weighs more is refused (422,
`session_full`). The size is checked before the request runs, so the requests already accepted can go past the cap.

## Docker

The root of the project has a `Dockerfile`, a `compose.yml` and the `service.yml` they use. From the root:

```
docker compose up -d                          # builds the image, starts the service on port 8080
curl localhost:8080/health                    # { "status": "ok" }
docker compose down                           # stop (add -v to forget the sessions too)
```

To run the first example through the service, from `examples/01-hello`, with Ollama on your computer (inside the
container `localhost` is the container itself, so the graph must name `host.docker.internal`):

```
sed 's/localhost/host.docker.internal/' graph.yml \
  | jq -R -s '{ yml: ., message: "What is a mutex?" }' \
  | curl -s localhost:8080/run -H 'content-type: application/json' -d @-
```

What `compose.yml` does, and what to change:

- **The network.** The services sit on a Docker network of their own, `a-graph`. Another container on that network
  reaches the service as `http://a-graph:8080` (`docker run --network a-graph_a-graph ...`, or add your container to
  `compose.yml`). The port is published on your computer too: the service does not authenticate its callers, so do
  not open it to others.
- **The port.** If 8080 is taken on your computer, choose another one: `A_GRAPH_PORT=9090 docker compose up -d`. The
  service always listens on 8080 inside the container and inside the network.
- **The configuration** is `service.yml` in the root, mounted read-only at `/etc/a-graph/service.yml`. Compose
  refuses to start if the file is missing. To change it for your computer only, copy it to `service.local.yml` and
  point Compose at the copy with a `compose.override.yml` (both files are ignored by git, and Compose reads the
  override by itself):

  ```yaml
  configs:
    service:
      file: ./service.local.yml
  ```
- **Ollama stays on your computer** by default, it is not in a container (on macOS a container would have no GPU).
  A graph reaches it as `http://host.docker.internal:11434`, and `service.yml` allows `host.docker.internal:11434`,
  that port only: the other services of your computer stay out of reach. Remove the line all the same when the
  service is not for you alone.
- **Ollama in a container**, on a Linux machine: `docker compose --profile ollama up -d`, then
  `docker compose exec ollama ollama pull qwen3.5:4b` once (the models stay in the volume `ollama-models`). A graph
  reaches it as `http://ollama:11434`.
- **The keys** of the cloud models are environment variables of the container, named in `compose.yml` and taken from
  your shell or from a `.env` file next to it: `ANTHROPIC_API_KEY=sk-... docker compose up -d`. The `allowed_env`
  of `service.yml` lets a graph send `ANTHROPIC_API_KEY` to `api.anthropic.com` and `OPENAI_API_KEY` to
  `api.openai.com` only: a variable you add to the container is out of reach of the graphs until you list it there.
  Its `allowed_ambient` is empty: no graph may use the cloud credentials of the container.
- **The sessions** live in the named volume `a-graph-data`. Do not replace it with a folder of yours: the container
  writes as user 10001 and, on Linux, the folder would refuse it. The `service.yml` of the repository deletes a
  session nobody used for 7 days and refuses one over 100 MB (`session_retention_seconds`, `session_max_bytes`):
  change the two values, or remove the lines to keep every session for ever.

Without Compose, the same thing is:

```
docker build -t a-graph .
docker run -p 8080:8080 \
  -v "$PWD/service.yml:/etc/a-graph/service.yml:ro" -v a-graph-data:/data \
  -e ANTHROPIC_API_KEY -e OPENAI_API_KEY \
  a-graph
```

Run it from the root of the project: if `service.yml` is not in the folder, Docker creates an empty directory with
that name and the service stops with ``a-graph: cannot read `/etc/a-graph/service.yml`: Is a directory (os error 21)``. The image holds only the
program. A container does not inherit the variables of your terminal: name each one with `-e NAME` (the value is
taken from your shell).
