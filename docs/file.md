# The graph file

A graph is one YAML file. This page explains the rules that hold for the whole file. Each key is described in
[the reference](graph.md).

## The blocks

| Block | Required | What it holds |
|---|---|---|
| `graph_spec` | yes | the version of the format: `"1.0"` |
| `passport` | yes | the identity of the graph, in the [ADL](https://www.adl-spec.org/) format, and the hosts it may reach |
| `observability` | yes | the log |
| `trace` | no | the timeline of the run, sent to a collector |
| `verbose` | yes | the live display of the run |
| `models` | yes | the language models |
| `embeddings` | no | the models that turn a text into a vector |
| `utilities` | no | programs and services that change the data of a node |
| `resources` | no | `prompts`, `tools`, `mcp_servers`, `media`, `skills` |
| `external` | no | `vectordb`, `search`, `memory` |
| `blackboards` | no | Markdown notes shared by nodes |
| `graph` | yes | the `nodes` and the `edges` that connect them |

The blocks may be written in any order.

## Keys

Only the keys described in the reference are accepted. A key that is not allowed where it is written is an error
(`unexpected_field`), and so is a mandatory key that is missing (`missing_field`) or a value of the wrong kind
(`bad_value`).

## Ids

Everything that is cited by other parts of the file has an `id`: models, embeddings, utilities, prompts, tools, MCP
servers, media, skills, vector databases, search engines, memories, blackboards and nodes. An id is unique in the
whole file, whatever it names: a model and a node cannot both be called `writer` (`duplicate_id`). Citing an id that
is not declared, or that names something of another kind, is an error (`unknown_ref`).

## Secrets

A secret is never written in the file. The file holds only the name of an environment variable, and the value is
read from the environment:

```yaml
api_key: { env: ANTHROPIC_API_KEY }
```

A secret written as text, `"${ANTHROPIC_API_KEY}"` included, is an error (`bad_value`): there is no substitution of
variables inside texts.

A variable that is not set is an error (`env_missing`). When it is reported depends on what uses it:

- the keys of the models, and the headers of tools, utilities, the log and the trace: when the file is checked,
  before anything runs;
- the key of a vector database, and the `env` and `headers` of an MCP server: when they are used, during the run.

When the file is checked, the variables are read only after the [destinations](#network): a file that names a host
it may not reach is reported with `network_not_allowed` alone, and its variables are not read. The headers of the
log are the exception: the log is built first, so they are read before the other destinations are checked.

## Network

The graph may reach only the hosts listed in `passport.adl.permissions.network.allowed_hosts`. Without
`permissions`, no host is allowed.

The list holds host names, not URLs: `api.anthropic.com`, not `https://api.anthropic.com/v1`. Hosts are compared
without the scheme and the path, and without regard to case. A local server is a host too: a model served at
`http://localhost:11434` needs `localhost` in the list.

An entry can also name a port, as `host:port` (`localhost:11434`; an IPv6 address in brackets, as `[::1]:8080`).
An entry without a port allows every port of the host. An entry with a port allows that port only. The port of a
destination is the one written in its URL, or the one of its scheme (80 for `http`, 443 for `https`). The host of a
provider (`api.anthropic.com`, the Bedrock and S3 hosts) is reached on 443. A URL whose scheme has no known port is
allowed only by an entry without a port. A port that is not a number from 1 to 65535 is an error (`bad_value`).

An entry is read as the host of a URL is read, so both sides of the check are written the same way: `127.1` counts
as `127.0.0.1`, `[0:0:0:0:0:0:0:1]` as `[::1]`, and a domain is lowercase. An entry that is a URL (`https://api.example.com`)
or carries a path (`api.example.com/v1`) is an error (`bad_value`).

Redirects are not followed: an answer that sends the call to another place is reported like any other failed
answer, with its status.

Every destination written in the file is checked, before anything runs:

- the host of every model and embedding provider, and for a `bedrock` model with a `batch` block also the Bedrock
  control API and the S3 buckets of the batch (see [models](models.md#batch));
- the `url` of utilities, tools and MCP servers with `transport: http`;
- the `url` of prompts, media and blackboards read from the network;
- the `url` of vector databases and search engines;
- `observability.output.url` and `trace.endpoint`.

A destination whose host and port are not allowed by the list is an error (`network_not_allowed`). The standard
output, files and the verbose display are not network.

The errors of the check are also written in the log. An `observability.output.url` that is not allowed is checked
first, alone: the check stops there and reports only that error, so the url is never called, not even to log it.

The check covers the destinations the file names. It does not cover the traffic that the SDKs of the cloud
providers make to resolve `credentials: ambient` (metadata servers, token endpoints).
