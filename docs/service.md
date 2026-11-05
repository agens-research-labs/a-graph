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
| `data_dir` | yes | text | The folder of the sessions. Each session has its own folder, `<data_dir>/<session>/`, and every path of a graph sent to the service is moved inside it. It should be a folder of the service alone, not one that holds other things. |
| `allowed_hosts` | yes | list of: text | The only hosts a graph may reach, compared without regard to case. An entry `host` allows every port of the host; an entry `host:port` allows that port only. An entry is read as the host of a URL is read (`127.1` counts as `127.0.0.1`); an entry that is a URL or carries a path is refused. The `allowed_hosts` of the passport of a graph can only be a part of this list: an entry of the graph is accepted when this list has the same entry or the host alone. A graph that names another host, or a host without a port that this list allows only on a port, is refused. |
| `allowed_env` | yes | map of names to: list of: text | The only environment variables of the service a graph may name with `{ env: VAR }`, each with the hosts its value may be sent to, written as the entries of `allowed_hosts` and allowed by it. A graph that names another variable, or sends a variable to a host not in its list, is refused (`env_not_allowed`), and the variable is not read: whether it is set is never told. `{}` allows none. A variable without hosts, or with a host that `allowed_hosts` does not allow, keeps the service from starting. |
| `allowed_ambient` | yes | list of: `bedrock`, `vertex_ai` or `azure_foundry` | The providers whose models a graph may run with `credentials: ambient`: the credentials of the service itself (its AWS role or variables, its Google Application Default Credentials, its Azure managed identity or CLI login), resolved by the SDK, not named by the graph. A graph with an `ambient` model of another provider is refused (`ambient_not_allowed`); a `vertex_ai` model is always `ambient`. With `azure_foundry` the service may start the programs `az` and `azd` to get a token. `[]` allows none. |
| `retention_seconds` | no | whole number | How long a run is kept after it ends, so that it can still be watched and asked for; then it is forgotten. The default is 600, at least 1. |
| `max_runs` | no | whole number | The most runs going on at once, those of `POST /run` and of `POST /runs` together. A request beyond it is refused with 503 (`too_many_runs`) before it is checked, not queued. The default is 8, at least 1. |
| `run_timeout_seconds` | no | whole number | The longest a run may last. Then it is stopped and ends as failed (`run_timeout`). The default is 3600, at least 1. |
| `session_retention_seconds` | no | whole number | How long a session folder is kept after its last request. Then the whole folder is deleted (its memories, blackboards and media), unless a run is still using it. Without this key nothing is ever deleted. At least 1. |
| `session_max_bytes` | no | whole number | The most a session folder may weigh, in bytes. A request for a session that weighs more is refused (422, `session_full`), and a refused request does not count as a use of the session for `session_retention_seconds`. It is checked before the request runs, so the requests already accepted can go past it. Without this key there is no cap. At least 1. |
