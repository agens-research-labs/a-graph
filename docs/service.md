# The service file

The keys of the file that starts the service, `a-graph --serve service.yml`. This page is generated from the code by
`a-graph --print-service-docs`: do not edit it by hand. The same keys are in the JSON Schema
`spec/service.schema.json`, printed by `a-graph --print-service-schema`.

The file holds no keys of models or services: those are variables of the environment of the service, which a graph
names as always, `api_key: { env: ANTHROPIC_API_KEY }`. The HTTP API of the service is described, in OpenAPI 3.1, in
`spec/service.openapi.json`, printed by `a-graph --print-service-api`.

## The file

| Key | Required | Value | Description |
|---|---|---|---|
| `port` | yes | whole number | The port the service listens on, on all interfaces. |
| `data_dir` | yes | text | The folder of the sessions. Each session has its own folder, `<data_dir>/<session>/`, and every path of a graph sent to the service is moved inside it. |
| `allowed_hosts` | yes | list of: text | The only hosts a graph may reach, compared without regard to case. An entry `host` allows every port of the host; an entry `host:port` allows that port only. An entry is read as the host of a URL is read (`127.1` counts as `127.0.0.1`); an entry that is a URL or carries a path is refused. The `allowed_hosts` of the passport of a graph can only be a part of this list: an entry of the graph is accepted when this list has the same entry or the host alone. A graph that names another host, or a host without a port that this list allows only on a port, is refused. |
| `retention_seconds` | no | whole number | How long a run is kept after it ends, so that it can still be watched and asked for; then it is forgotten. The default is 600, at least 1. |
| `max_runs` | no | whole number | The most runs going on at once, those of `POST /run` and of `POST /runs` together. A request beyond it is refused with 503 (`too_many_runs`), not queued. The default is 8, at least 1. |
