# The reference

Every key of the graph file. This page is generated from the code by `a-graph --print-docs`: do not edit it by hand.
The same keys are in the JSON Schema `spec/graph.schema.json`, printed by `a-graph --print-schema`.

How the file works as a whole is explained in [the graph file](file.md). In the paths, `[]` is an item of a list and
`<name>` a key you choose. A secret is written `{ env: <VARIABLE> }`: the name of the environment variable that holds
its value.

## The file

| Key | Required | Value | Description |
|---|---|---|---|
| `graph_spec` | yes | `1.0` | The version of the format of this file. |
| `passport` | yes | map (below) | The identity of the graph, in the ADL 0.3.0 format (https://www.adl-spec.org/), with the hosts it may reach. |
| `observability` | yes | map (below) | The log: structured records of what the program does, for machines. |
| `trace` | no | map (below) | The timeline of the run, sent to a collector when the run ends. Without this block there is no trace. |
| `verbose` | yes | map (below) | The live display of the run, for a person, on the standard error. |
| `models` | yes | list of: map (below) | The language models the nodes call. |
| `embeddings` | no | list of: map (below) | The models that turn a text into a vector, for the vector databases and the memories. |
| `utilities` | no | list of: map (below) | Programs and services that the program, never the model, calls to change the data of a node. |
| `resources` | no | map (below) | What the nodes use: prompts, tools, MCP servers, media, skills. |
| `external` | no | map (below) | The sources outside the program: vector databases, search engines, memories. |
| `blackboards` | no | list of: map (below) | Markdown texts shared by the nodes. |
| `graph` | yes | map (below) | The nodes and the edges that connect them. |

## `passport`

| Key | Required | Value | Description |
|---|---|---|---|
| `adl` | yes | map (below) | The passport, in the ADL format (https://www.adl-spec.org/). The key names are those of ADL. |

## `passport.adl`

| Key | Required | Value | Description |
|---|---|---|---|
| `adl_spec` | yes | text | The version of ADL the passport is written in: `"0.3.0"`. |
| `name` | yes | text | The name of the agent. The trace uses it as the name of the service. |
| `description` | yes | text | What the agent does. |
| `version` | yes | text | The version of the agent, as a semantic version of three numbers (for example `"1.0.0"`). |
| `data_classification` | yes | map (below) | How sensitive the data of the graph are. It decides whether the contents (prompts, answers, results) are written in the log, the trace and the display. |
| `lifecycle` | no | map (below) | The stage of life of the agent. The program does not act on it. |
| `permissions` | no | map (below) | What the graph is allowed to do. Without it, no host is allowed. |

## `passport.adl.data_classification`

| Key | Required | Value | Description |
|---|---|---|---|
| `sensitivity` | yes | `public`, `internal`, `confidential` or `restricted` | The level of sensitivity of the data. The contents are written only with `public` and no `categories`. |
| `categories` | no | list of: `pii`, `phi`, `financial`, `credentials`, `intellectual_property` or `regulatory` | The kinds of regulated data the graph handles. Any category hides the contents. |

## `passport.adl.lifecycle`

| Key | Required | Value | Description |
|---|---|---|---|
| `status` | yes | `draft`, `active`, `deprecated` or `retired` | The stage the agent is in. |
| `sunset_date` | no | text | When the agent is retired, in RFC 3339 with a time zone (for example `2027-01-01T00:00:00Z`). |

## `passport.adl.permissions`

| Key | Required | Value | Description |
|---|---|---|---|
| `network` | no | map (below) | The network the graph may reach. Without it, no host is allowed. |

## `passport.adl.permissions.network`

| Key | Required | Value | Description |
|---|---|---|---|
| `allowed_hosts` | yes | list of: text | The hosts the graph may reach: host names, not URLs (`api.anthropic.com`, not `https://api.anthropic.com/v1`), compared without regard to case. An entry `host` allows every port of the host; an entry `host:port` (`localhost:11434`, `[::1]:8080`) allows that port only. The port of a URL is the one written in it, or 80 for `http` and 443 for `https`; the host of a provider is reached on 443. An entry is read as the host of a URL is read: `127.1` counts as `127.0.0.1`, `[0:0:0:0:0:0:0:1]` as `[::1]`. An entry that is a URL or carries a path is refused. Every destination of the file is checked against this list before anything runs. |

## `observability`

| Key | Required | Value | Description |
|---|---|---|---|
| `level` | yes | `debug`, `info`, `warn` or `error` | The lowest level written: `info` writes the run, the nodes and the groups, `debug` also the calls, `warn` and `error` only the problems. |
| `output` | no | `stdout` or `stderr`, or map (below) | Where the log goes: `stderr` (the default), with the errors, so that the standard output holds only the result; `stdout`, mixed with the result; a file; or an HTTP endpoint. A log that cannot be written never stops the run. |

## `observability.output`

The form with `file`:

| Key | Required | Value | Description |
|---|---|---|---|
| `file` | yes | text | The file the log is added to, created if it does not exist. |

The form with `url` and `headers`:

| Key | Required | Value | Description |
|---|---|---|---|
| `url` | yes | text | The HTTP endpoint the log is sent to, as POSTs of `application/x-ndjson` with at most 100 events each. |
| `headers` | no | map of names to: secret | The headers of the POSTs, each read from an environment variable. |

## `trace`

| Key | Required | Value | Description |
|---|---|---|---|
| `endpoint` | yes | text | The OpenTelemetry collector, as the exact URL of the POST (for example `http://host:4318/v1/traces`). At the end of the run, also when it fails, the whole timeline is sent in one POST, in OTLP over HTTP with JSON. A POST that fails is tried again up to 3 times; what cannot be delivered is dropped, with a notice on the standard error. |
| `headers` | no | map of names to: secret | The headers of the POST, each read from an environment variable. |
| `capture_content` | no | `true` or `false` | Whether trace attributes include prompts, answers, reasoning and error details. The default is false. Enabling this never overrides passport redaction; logs and the live display retain their own behavior. |
| `max_content_bytes` | no | whole number | The most UTF-8 bytes of each trace content attribute, at least 1; the default is 4096. Truncation preserves character boundaries and adds a_graph.content_truncated=true. |

## `verbose`

| Key | Required | Value | Description |
|---|---|---|---|
| `level` | yes | `off`, `steps` or `detail` | What is shown: `off` nothing; `steps` the run, the nodes and the groups as they start and end, and the errors and warnings; `detail` also the calls, the readings and the contents, as the passport allows. |

## `models[]`

The key `provider` chooses the other keys, described in a section for each value.

| Key | Required | Value | Description |
|---|---|---|---|
| `provider` | yes | `anthropic`, `openai`, `gemini`, `mistral`, `bedrock`, `vertex_ai`, `azure_foundry`, `ollama` or `custom` | The service the model is reached through. It decides the other keys. |
| `id` | yes | the id of this model | The id of the model, cited by the nodes. |
| `model` | yes | text | The name of the model at its provider; on `azure_foundry`, the name of the deployment created in the Foundry portal. |
| `input` | yes | list of: `text`, `image`, `audio` or `video` | What the model reads. The program does not check the nodes against this list: what a node sends is checked against what the provider accepts. |
| `output` | yes | list of: `text`, `image`, `audio` or `video` | What the model produces. On `bedrock`, and with publisher `anthropic`, anything but `text` is an error (`output_not_text`). Otherwise the program does not check the nodes against this list: what a node asks is checked against what the provider can produce. |

## `models[]` with `provider: anthropic`

The Anthropic Messages API (`api.anthropic.com`). A node that uses it must set `max_tokens`.

| Key | Required | Value | Description |
|---|---|---|---|
| `api_key` | yes | secret | The key of the API, read from an environment variable when the file is checked. |
| `workspace_id` | no | text | The Anthropic workspace, sent as the header `anthropic-workspace-id`, for a key that is not bound to a workspace. It is an id, not a secret. |

## `models[]` with `provider: openai`

The OpenAI Responses API (`api.openai.com`).

| Key | Required | Value | Description |
|---|---|---|---|
| `api_key` | yes | secret | The key of the API, read from an environment variable when the file is checked. |

## `models[]` with `provider: gemini`

The Gemini API with an API key, not Vertex AI (`generativelanguage.googleapis.com`). It has no batch, and reasoning cannot be switched off.

| Key | Required | Value | Description |
|---|---|---|---|
| `api_key` | yes | secret | The key of the API, read from an environment variable when the file is checked. |

## `models[]` with `provider: mistral`

The Mistral API (`api.mistral.ai`).

| Key | Required | Value | Description |
|---|---|---|---|
| `api_key` | yes | secret | The key of the API, read from an environment variable when the file is checked. |

## `models[]` with `provider: bedrock`

Amazon Bedrock, through the Converse API (`bedrock-runtime.<region>.amazonaws.com`). It produces text only.

| Key | Required | Value | Description |
|---|---|---|---|
| `region` | yes | text | The AWS region the model runs in, for example `eu-west-1`. |
| `credentials` | yes | `ambient`, or map (below) | `ambient`: the default chain of the AWS SDK, resolved at the first call; the traffic it needs is not checked against `allowed_hosts`. Or the keys, each read from an environment variable when the file is checked. |
| `batch` | no | map (below) | Needed only by a `batch` group. The job is created on the Bedrock control API (`bedrock.<region>.amazonaws.com`), which must be in `allowed_hosts`. A batch cannot have an output `schema`. |

## `models[].credentials` with `provider: bedrock`

| Key | Required | Value | Description |
|---|---|---|---|
| `access_key` | yes | secret | The AWS access key id. |
| `secret_key` | yes | secret | The AWS secret access key. |
| `session_token` | no | secret | The session token, only for temporary credentials. |

## `models[].batch` with `provider: bedrock`

| Key | Required | Value | Description |
|---|---|---|---|
| `input_s3_uri` | yes | text | The S3 folder the requests are written to, as `s3://<bucket>/<prefix>`. Its bucket must be in `allowed_hosts`. |
| `output_s3_uri` | yes | text | The S3 folder the answers are written to, as `s3://<bucket>/<prefix>`. Its bucket must be in `allowed_hosts`. |
| `role_arn` | yes | text | The IAM role the batch job runs with. |

## `models[]` with `provider: vertex_ai`

Google Cloud Vertex AI. With publisher `anthropic` it works as `anthropic` and produces text only; with `google` as `gemini`. Neither has a batch yet.

| Key | Required | Value | Description |
|---|---|---|---|
| `publisher` | yes | `anthropic` or `google` | Whose model it is: it decides the format of the requests and what the model can do. |
| `project` | yes | text | The Google Cloud project. |
| `location` | yes | text | Where the model runs: `global` (`aiplatform.googleapis.com`), `us` or `eu` (`aiplatform.<location>.rep.googleapis.com`), or a region (`<location>-aiplatform.googleapis.com`). |
| `credentials` | yes | `ambient` | The Google Application Default Credentials, resolved at the first call; the traffic they need is not checked against `allowed_hosts`. |
| `batch` | no | map (below) | Accepted only with publisher `google` (`batch_not_allowed`), whose batch is not available yet. |

## `models[].batch` with `provider: vertex_ai`

| Key | Required | Value | Description |
|---|---|---|---|
| `input_uri` | yes | text | Where the requests of the batch are read from. |
| `output_uri_prefix` | yes | text | Where the answers of the batch are written. |

## `models[]` with `provider: azure_foundry`

Microsoft Foundry. With publisher `anthropic` it works as `anthropic`, produces text only and has no batch; with `openai` as `openai`.

| Key | Required | Value | Description |
|---|---|---|---|
| `publisher` | yes | `anthropic` or `openai` | Whose model it is: it decides the format of the requests and what the model can do. |
| `resource` | yes | text | The name of the Foundry resource: the host is `<resource>.services.ai.azure.com` for `anthropic`, `<resource>.openai.azure.com` for `openai`. |
| `credentials` | yes | `ambient`, or map (below) | `ambient`: in this order, the managed identity of an Azure resource, the Azure CLI and the Azure Developer CLI, resolved at the first call; the traffic they need is not checked against `allowed_hosts`. Or an `api_key`. |
| `batch` | no | map (below) | Accepted only with publisher `openai` (`batch_not_allowed`). |

## `models[].credentials` with `provider: azure_foundry`

| Key | Required | Value | Description |
|---|---|---|---|
| `api_key` | yes | secret | The key of the resource, read from an environment variable when the file is checked. |

## `models[].batch` with `provider: azure_foundry`

| Key | Required | Value | Description |
|---|---|---|---|
| `deployment` | yes | text | The Global-Batch deployment the batch is sent to. |

## `models[]` with `provider: ollama`

An Ollama server.

| Key | Required | Value | Description |
|---|---|---|---|
| `host` | yes | text | The URL of the server, for example `http://localhost:11434`. Its host must be in `allowed_hosts`. |

## `models[]` with `provider: custom`

A model server of your own, called with `POST <host>/api/chat` in the format of the Ollama API.

| Key | Required | Value | Description |
|---|---|---|---|
| `host` | yes | text | The URL of the server, for example `http://localhost:11434`. Its host must be in `allowed_hosts`. |

## `embeddings[]`

The key `provider` chooses the other keys, described in a section for each value.

| Key | Required | Value | Description |
|---|---|---|---|
| `provider` | yes | `ollama` | The service the model is reached through. It decides the other keys. |
| `id` | yes | the id of this embedding | The id of the embedding model, cited by the vector databases and the memories. |
| `model` | yes | text | The name of the model at its provider. |

## `embeddings[]` with `provider: ollama`

An Ollama server, called with `POST <host>/api/embed`.

| Key | Required | Value | Description |
|---|---|---|---|
| `host` | yes | text | The URL of the server, for example `http://localhost:11434`. Its host must be in `allowed_hosts`. |

## `utilities[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this utility | The id of the utility, cited by the nodes where they want their data changed. |
| `transport` | yes | `stdio` or `http` | `stdio`: the program `command` is started for each call. `http`: a service already running, which the program does not start or stop. |
| `command` | no | text | The program to start, with `transport: stdio` only (where it is mandatory). |
| `args` | no | list of: text | The arguments of `command`, with `transport: stdio` only. |
| `url` | no | text | The URL the calls are POSTed to, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`. |
| `headers` | no | map of names to: secret | The headers of the POSTs, with `transport: http` only, each read from an environment variable when the file is checked. |
| `timeout` | yes | whole number | The seconds a call may take, at least 1. After that the program is stopped or the request abandoned, and the run fails. |

## `resources`

| Key | Required | Value | Description |
|---|---|---|---|
| `prompts` | no | list of: map (below) | The texts the nodes send to their models, with their placeholders. |
| `tools` | no | list of: map (below) | Programs and services of yours that the model may call while a node runs. The model decides whether to call them; the program makes the call. |
| `mcp_servers` | no | list of: map (below) | Servers of the Model Context Protocol, connected when the run starts. Only their tools are used, never their resources or prompts. The arguments of a call are checked against the input schema the server gives for the tool, as for `tools`. |
| `media` | no | list of: map (below) | Everything that is not the text of a message: documents, images, audio, read or produced by the nodes. |
| `skills` | no | list of: map (below) | Folders in the Agent Skills format (https://agentskills.io/specification): instructions for a task, with their files and scripts, that the model reads when it needs them. |

## `resources.prompts[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this prompt | The id of the prompt, cited by the nodes. |
| `content` | no | map (below) | The texts of the prompt, written in the file. A prompt has exactly one of `content` and `source`. |
| `source` | no | map (below) | Where the prompt is read from: exactly one of `path`, a `.yml` file with the same `content: { system, user }` (not Markdown, which could not be split into the two texts), and `url`. |

## `resources.prompts[].content`

| Key | Required | Value | Description |
|---|---|---|---|
| `system` | yes | text | The system text, with its placeholders. |
| `user` | yes | text | The user text, with its placeholders. |

## `resources.prompts[].source`

| Key | Required | Value | Description |
|---|---|---|---|
| `path` | no | text | A file, relative to the folder the command is run from. |
| `url` | no | text | A URL. It is not read yet: a node that uses it fails when it runs. |
| `access` | no | any value | How to get in at the `url`, with `url` only. Its content is not defined yet. |
| `base64` | no | text | The content itself, in base64, checked when the graph loads. Only for a media. |
| `mime_type` | no | text | The type of the content in `base64`, such as `application/pdf`, given to the utilities the media goes through. Only for a media, with `base64`. |

## `resources.tools[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this tool | The id of the tool: the name the model calls it by, and the id the nodes cite. |
| `description` | yes | text | What the tool does, told to the model. |
| `transport` | yes | `stdio` or `http` | `stdio`: the program `command` is started for each call, gets the arguments on its standard input and writes the result on its standard output. `http`: the arguments are POSTed to `url` and the result is the body of the answer. |
| `command` | no | text | The program to start, with `transport: stdio` only (where it is mandatory). |
| `args` | no | list of: text | The arguments of `command`, with `transport: stdio` only. |
| `url` | no | text | The URL the calls are POSTed to, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`. |
| `headers` | no | map of names to: secret | The headers of the POSTs, with `transport: http` only, each read from an environment variable when the file is checked. |
| `max_iter` | yes | whole number | The most calls of this tool in one run of a node, at least 1, failures included. The count starts again for each node and each appearance of the node in the edges; one more call fails the run (`tool_limit`). |
| `timeout` | yes | whole number | The seconds one call may take, at least 1. After that the program is killed or the request abandoned, and the model is told, as for any other failure. |
| `parameters` | yes | any value | The JSON Schema of the arguments, an object, given to the provider as it is written; it must be a valid JSON Schema (`bad_value`). Before a call the arguments the model chose are checked against it: if they do not match, the tool is not called and the model is told why, as for any other failure, and the call counts in `max_iter`. |

## `resources.mcp_servers[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this MCP server | The id of the server, cited by the nodes. |
| `transport` | yes | `stdio` or `http` | `stdio`: the program `command` is started when the run starts. `http`: the server at `url`, reached with Streamable HTTP. |
| `command` | no | text | The program to start, with `transport: stdio` only (where it is mandatory). |
| `args` | no | list of: text | The arguments of `command`, with `transport: stdio` only. |
| `env` | no | map of names to: secret | The environment variables of the program, with `transport: stdio` only, each read from an environment variable when the server is started. |
| `url` | no | text | The URL of the server, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`. |
| `headers` | no | map of names to: secret | The headers of the requests, with `transport: http` only, each read from an environment variable when the server is connected. |
| `max_iter` | yes | whole number | The most calls to the tools of the server, any of them, in one run of a node, at least 1. One more call fails the run (`tool_limit`). |
| `timeout` | yes | whole number | The seconds one call may take, at least 1. After that the model is told that the call failed. |

## `resources.media[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this media | The id of the media, cited by the nodes that read or produce it. |
| `type` | yes | `md` or `file`, or map (below) | What the media is: `md` a text (a file of another format, such as a PDF, is declared `md` and read through a utility that changes it into text); `file` a file of any format made by a utility, delivered in base64; or an image, an audio or a video with its format. |
| `source` | no | map (below) | Where a media that is read comes from: exactly one of `path`, `url` and `base64`. A media has exactly one of `source` and `generate`, except one of type `file`, which has neither. |
| `generate` | no | map (below) | How a model makes the media. Every value is given to the provider as it is written; a value it does not accept is an error when the file is checked (`not_supported`). A media is one image or one audio, written by one node only (`media_two_writers`). |
| `provider` | no | map (below) | Settings of one family of providers only (`exactly_one`): the family of the model of the node that writes the media. |

## `resources.media[].type`

A map with exactly one of these keys.

| Key | Value | Description |
|---|---|---|
| `img` | text | The format of the image: `png`, `jpeg`, `gif`, `webp`... |
| `audio` | text | The format of the audio: `wav`, `mp3`... |
| `video` | text | The format of the video: `mp4`... |

## `resources.media[].source`

| Key | Required | Value | Description |
|---|---|---|---|
| `path` | no | text | A file, relative to the folder the command is run from. |
| `url` | no | text | A URL. It is not read yet: a node that uses it fails when it runs. |
| `access` | no | any value | How to get in at the `url`, with `url` only. Its content is not defined yet. |
| `base64` | no | text | The content itself, in base64, checked when the graph loads. Only for a media. |
| `mime_type` | no | text | The type of the content in `base64`, such as `application/pdf`, given to the utilities the media goes through. Only for a media, with `base64`. |

## `resources.media[].generate`

| Key | Required | Value | Description |
|---|---|---|---|
| `voice` | no | text | The voice of an audio. Mandatory for an audio on OpenAI. |
| `language_code` | no | text | The language of an audio. No provider accepts it at present. |
| `size` | no | map (below) | The size of an image. The provider may accept only some pairs. |
| `quality` | no | text | The quality of an image. The values depend on the model. |
| `background` | no | `true` or `false` | `true` a transparent background, `false` an opaque one; without it the provider decides. A transparent background needs an image of type `png` or `webp` (`media_type_unsupported`). |

## `resources.media[].generate.size`

| Key | Required | Value | Description |
|---|---|---|---|
| `width` | yes | whole number | The width, in pixels. |
| `height` | yes | whole number | The height, in pixels. |

## `resources.media[].provider`

| Key | Required | Value | Description |
|---|---|---|---|
| `openai` | no | map (below) | For a model of OpenAI, also through `azure_foundry` with publisher `openai`. |
| `gemini` | no | map (below) | For a model of Gemini, also through `vertex_ai` with publisher `google`. |

## `resources.media[].provider.openai`

| Key | Required | Value | Description |
|---|---|---|---|
| `moderation` | no | `low` or `auto` | How strict the moderation of the image is. |
| `output_compression` | no | whole number | From 0 to 100, for an image of type `jpeg` or `webp`. |
| `style` | no | `vivid` or `natural` | The style of the image. No model accepts it at present: the GPT image models take the style from the prompt. Not together with the other two (`media_provider_conflict`). |

## `resources.media[].provider.gemini`

| Key | Required | Value | Description |
|---|---|---|---|
| `image_size` | no | `512`, `1K`, `2K` or `4K` | The size of the image. |
| `aspect_ratio` | no | text | The ratio of width to height, for example `16:9`. |

## `resources.skills[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this skill | The id of the skill, cited by the nodes. |
| `path` | yes | text | The folder of the skill, with its `SKILL.md`. Its name and description are written in `SKILL.md`, not here. |
| `run_scripts` | no | `true` or `false` | Whether the model may run the scripts in the folder `scripts/` of the skill. The default is `true`. |
| `max_iter` | no | whole number | The most reads of files and runs of scripts together, for this skill, in one run of a node. The default is 20, at least 1; the activation is not counted. One more fails the run (`tool_limit`). |
| `timeout` | no | whole number | The seconds a script may take. The default is 30, at least 1. |
| `max_files` | no | whole number | The most files listed when the skill is activated. The default is 200, at least 1. |
| `max_file_bytes` | no | whole number | The largest file the model may read, and the most bytes kept of the output of a script. The default is 100000, at least 1. |
| `max_depth` | no | whole number | The most levels of sub-folders listed when the skill is activated. The default is 6, at least 1. |

## `external`

| Key | Required | Value | Description |
|---|---|---|---|
| `vectordb` | no | list of: map (below) | Vector databases the nodes search: what they find becomes `{{chunks}}` in the prompt. |
| `search` | no | list of: map (below) | Search engines that are not vector databases. They cannot run yet: a node that uses one fails when it runs. |
| `memory` | no | list of: map (below) | Memories that keep the conversation of a node between runs. |

## `external.vectordb[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this vectordb | The id of the vector database, cited by the nodes and by the memories. |
| `provider` | yes | `qdrant` or `chromadb` | The kind of database. |
| `url` | no | text | The URL of the database. Mandatory; its host must be in `allowed_hosts`. |
| `api_key` | no | secret | The key of the database, read from an environment variable when it is used, and sent as the header `api-key` (`qdrant`) or `x-chroma-token` (`chromadb`). |
| `collection` | no | text | The collection that is searched. Mandatory. |
| `top_k` | no | whole number | How many chunks a search gives back. The default is 5. |
| `timeout` | no | whole number | The seconds a search may take, at least 1. The default is 60. |
| `embedding` | yes | the id of an embedding | The embedding model that turns the query into a vector. |
| `text_field` | no | text | The field of the payload that holds the text of a chunk. Mandatory with `qdrant`, not allowed with `chromadb`, which keeps the text in its documents. |
| `tenant` | no | text | The tenant, with `chromadb` only. The default is `default_tenant`. |
| `database` | no | text | The database, with `chromadb` only. The default is `default_database`. |

## `external.search[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this search | The id of the search engine, cited by the nodes. |
| `provider` | yes | text | The kind of search engine. |
| `url` | yes | text | The URL of the search engine. Its host must be in `allowed_hosts`. |
| `access` | no | any value | How to get in. Its content is not defined yet. |

## `external.memory[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this memory | The id of the memory, cited by the nodes. |
| `provider` | yes | `json_file` or `vectordb` | Where the memory is kept: `json_file`, a JSON file; `vectordb`, a vector database, which cannot run yet. |
| `path` | no | text | The JSON file, with `json_file` only (where it is mandatory). It is created at the first write. |
| `ref` | no | the id of a vectordb | The vector database, with `vectordb` only (where it is mandatory). |
| `synt` | no | map (below) | The summary, with `json_file` only. Without it the messages are never summarized, and the memory has no placeholder. |
| `chat` | no | map (below) | The history, with `json_file` only (where it is mandatory). |

## `external.memory[].synt`

| Key | Required | Value | Description |
|---|---|---|---|
| `length` | yes | whole number | How many exchanges the record of a node reaches before they are summarized, at least 1. |
| `model` | yes | the id of a model | The model that writes the summary. |
| `prompt` | no | the id of a prompt | The prompt of the summary, which must use both `{{synt}}` (the previous summary) and `{{chat}}` (the exchanges to summarize). Without it the program uses a prompt of its own. |
| `max_tokens` | no | whole number | The most tokens of the summary. The default is 4000, since a model that reasons spends part of them before it writes. |

## `external.memory[].chat`

| Key | Required | Value | Description |
|---|---|---|---|
| `length` | yes | whole number | How many exchanges become the history of a call, at least 1. |
| `embedding` | no | the id of an embedding | The embedding model that turns the messages of the user into vectors. With it, the history is the exchanges most similar to the new message, instead of the last ones. |

## `blackboards[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this blackboard | The id of the blackboard, cited by the nodes and used as the placeholder of its text. |
| `source` | yes | `memory`, or map (below) | Where the blackboard is kept: `memory`, in the program, empty at the start and lost at the end of the run; a file; or a URL. |

## `blackboards[].source`

The form with `file`:

| Key | Required | Value | Description |
|---|---|---|---|
| `file` | yes | text | A Markdown file, which must exist when the file is checked and start with its header: a `<description>` and a `<purpose>`, each followed by a text, between two `---` lines (`blackboard_header`). |

The form with `url` and `access`:

| Key | Required | Value | Description |
|---|---|---|---|
| `url` | yes | text | A URL. It is not supported yet: a node that uses it fails when it runs. |
| `access` | no | any value | How to get in. Its content is not defined yet. |

## `graph`

| Key | Required | Value | Description |
|---|---|---|---|
| `nodes` | yes | list of: map (below) | The nodes: each a model, what it uses, where its input comes from and where its output goes. |
| `edges` | yes | list of: a `step` (below) | The order in which the nodes run: a list read in order. |

## `graph.nodes[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of this node | The id of the node, cited by the edges. |
| `model` | no | map (below) | The model of the node and the settings of its calls. Mandatory, except on a validation node, which has it inside `validation-node`. |
| `external` | no | map (below) | The vector databases, search engines and memories the node uses. |
| `resources` | no | map (below) | The prompt, the tools, the MCP servers and the skills of the node. |
| `blackboard` | no | map (below) | The one blackboard the node manages. |
| `media` | no | map (below) | The media the node reads and produces. |
| `message` | no | map (below) | Where the text of the node comes from and where its answer goes. |
| `validation-node` | no | map (below) | Makes the node a validation node, which reads and judges and produces nothing; the node then has only `id` and `validation-node`. Approved, the message it examined goes on unchanged. It stands in the main flow only, never in a group or a `react`. |

## `graph.nodes[].model`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a model | The model the node calls. |
| `temperature` | no | number | Given to the provider as it is written. |
| `top_p` | no | number | Given to the provider as it is written. |
| `max_tokens` | no | whole number | The most tokens of the answer: the cap on the cost, set for each node. Mandatory with `anthropic`, and with publisher `anthropic` on `vertex_ai` and `azure_foundry`; without it the provider decides. |
| `reasoning` | no | `off`, `minimal`, `low`, `medium`, `high` or `max` | How much the model reasons, sent in the form of its provider; a level the provider does not have is replaced by the nearest one. Without it nothing is sent. `off` is an error on `gemini` and with publisher `google`. |

## `graph.nodes[].external`

| Key | Required | Value | Description |
|---|---|---|---|
| `vectordb` | no | list of: map (below) | The vector databases the node searches, in this order: what they find goes where the prompt writes `{{chunks}}`. |
| `search` | no | list of: map (below) | The search engines the node uses. They cannot run yet. |
| `memory` | no | list of: map (below) | The memories the node reads and writes. |

## `graph.nodes[].external.vectordb[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a vectordb | The vector database. |
| `query` | yes | text | What is searched: `user_message`, the message of the user, or a fixed text. |
| `utility` | no | map (below) | Utilities on the query and on the chunks. |

## `graph.nodes[].external.vectordb[].utility`

| Key | Required | Value | Description |
|---|---|---|---|
| `query` | no | the id of an utility, or list of: the id of an utility | Utilities that change the query, before it is turned into a vector. |
| `chunks` | no | the id of an utility, or list of: the id of an utility | Utilities that change each chunk found, before the prompt. |

## `graph.nodes[].external.search[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a search | The search engine. Its result goes where the prompt writes `{{<search-id>}}`. |
| `query` | yes | text | What is searched: `user_message`, the message of the user, or a fixed text. |

## `graph.nodes[].external.memory[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a memory | The memory. Its summary, if it has `synt`, goes where the prompt writes `{{<memory-id>}}`. |
| `read` | yes | map (below) | Which roles of the saved conversation come back as the history of the call. |
| `write` | yes | map (below) | Which roles of this exchange are saved. |

## `graph.nodes[].external.memory[].read`

| Key | Required | Value | Description |
|---|---|---|---|
| `user` | yes | `true` or `false` | The messages of the user. |
| `assistant` | yes | `true` or `false` | The answers of the node. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change, in `read`, each message read back and the summary; in `write`, what is saved, after the utilities of `message.output`. |

## `graph.nodes[].external.memory[].write`

| Key | Required | Value | Description |
|---|---|---|---|
| `user` | yes | `true` or `false` | The messages of the user. |
| `assistant` | yes | `true` or `false` | The answers of the node. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change, in `read`, each message read back and the summary; in `write`, what is saved, after the utilities of `message.output`. |

## `graph.nodes[].resources`

| Key | Required | Value | Description |
|---|---|---|---|
| `prompt` | no | the id of a prompt | The prompt of the node. Without it the model gets no system text and its user text is the message of the node, which needs exactly one of the two messages (`prompt_required`). |
| `tools` | no | list of: the id of a tool | The tools the model may call. |
| `mcp_servers` | no | list of: the id of an MCP server | The MCP servers whose tools the model may call. |
| `skills` | no | list of: the id of a skill | The skills the model may use. |
| `max_rounds` | no | whole number | The most rounds in which the model may ask for tools in one run of the node. The default is 20, at least 1. Every round counts, whatever it asks for: also a skill, or a tool the node does not have, which no `max_iter` counts. One more round fails the run (`tool_limit`). |

## `graph.nodes[].blackboard`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a blackboard | The blackboard. |
| `read` | no | `true` or `false` | `true`: its text goes where the prompt writes `{{<blackboard-id>}}`, which is then mandatory. The default is `false`. |
| `write` | no | map (below) | After the node answers, its text is added to the blackboard, separated by an empty line. |
| `cleanup` | no | `true` or `false` | `true`: when the node is done, everything below the header is removed. Not with an enabled `write` (`cleanup_write`). The default is `false`. |

## `graph.nodes[].blackboard.write`

| Key | Required | Value | Description |
|---|---|---|---|
| `enabled` | yes | `true` or `false` | Whether the node writes. |
| `place` | yes | `top` or `bottom` | Where the text is added: at the top or at the bottom of what is already below the header. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change the text written, after the utilities of `message.output`. |

## `graph.nodes[].media`

| Key | Required | Value | Description |
|---|---|---|---|
| `input` | no | list of: map (below) | The media the node reads. |
| `output` | no | list of: map (below) | The media the node produces. |

## `graph.nodes[].media.input[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a media | The media. |
| `from` | yes | `resources` or `node` | `resources`: a media with a `source`. `node`: a media produced by a node connected before (`media_not_from_node`); it arrives only if that node passes it on with `node: true`. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities the media goes through before the node uses it. A file on disk reaches a program as its path; everything else reaches a utility as its bytes. They must end with a text or an image. |

## `graph.nodes[].media.output[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a media | The media. |
| `from` | no | `model` or `message` | `model` (the default): the model makes it, so the media must have `generate`. `message`: a utility makes it from the text of the node, so the media must be of type `file` and `utility` is mandatory. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change the file the model made, or that make the file from the text of the node. They must end with a file. |
| `assistant` | no | `true` or `false` | `true`: the media goes to the final output. The default is `false`. |
| `node` | no | `true` or `false` | `true`: the media goes to the node connected after. The default is `false`. |

## `graph.nodes[].message`

| Key | Required | Value | Description |
|---|---|---|---|
| `input` | no | map (below) | Where the text of the node comes from. |
| `output` | no | map (below) | Where the answer of the node goes. |

## `graph.nodes[].message.input`

| Key | Required | Value | Description |
|---|---|---|---|
| `user_message` | no | `true` or `false` | `true`: the node takes the message of the user, the one given to the command. The default is `false`. |
| `agent_message` | no | `true` or `false` | `true`: the node takes what the node connected before sends on. The default is `false`. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change the messages the node takes, which are then used everywhere: in the prompt, in the queries and in what the memories save. A message the node does not take is not changed. |

## `graph.nodes[].message.output`

| Key | Required | Value | Description |
|---|---|---|---|
| `assistant_message` | no | `true` or `false` | `true`: the answer goes to the final output. The default is `false`. |
| `node` | no | `true` or `false` | `true`: the answer goes to the node connected after, as its `agent_message`. The default is `false`. |
| `reasoning` | no | `true` or `false` | `true`: the reasoning of the model goes to the final output too. It never goes to another node. The default is `false`. |
| `logprobs` | no | whole number | From 0 to 20: the log probability of every token of the answer goes to the final output too, with that many most likely alternatives for each token; on `ollama` and `custom` a model that reasons gives those of its reasoning too, before them. It never goes to another node. Only on `openai`, `mistral`, `ollama`, `custom` and `azure_foundry` with publisher `openai`. Without it nothing is asked. |
| `schema` | no | any value | A JSON Schema, written as JSON in a text or as YAML: the model answers in that shape, and the answer is a JSON value. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change the answer once, before every destination. |

## `graph.nodes[].validation-node`

| Key | Required | Value | Description |
|---|---|---|---|
| `model` | yes | map (below) | The model of the node and the settings of its calls. |
| `resources` | yes | map (below) | The prompt of the node, its only resource. |
| `external` | no | map (below) | The vector databases, search engines and memories the node reads. |
| `blackboard` | no | map (below) | The blackboard the node reads. |
| `media` | no | map (below) | The media the node reads. |
| `message` | no | map (below) | Where the text the node judges comes from. |
| `response` | no | any value | Accepted and not used: the answer of a validation node always has the shape `{ validation, reasons, log }`. |
| `on_error` | yes | `stop` or `node` | What a refusal does. `stop`: the run ends, and the final output gets the reasons and the log. `node`: the run goes on, and the next node receives the problem as its `agent_message`; it needs a node after it. |

## `graph.nodes[].validation-node.model`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a model | The model the node calls. |
| `temperature` | no | number | Given to the provider as it is written. |
| `top_p` | no | number | Given to the provider as it is written. |
| `max_tokens` | no | whole number | The most tokens of the answer: the cap on the cost, set for each node. Mandatory with `anthropic`, and with publisher `anthropic` on `vertex_ai` and `azure_foundry`; without it the provider decides. |
| `reasoning` | no | `off`, `minimal`, `low`, `medium`, `high` or `max` | How much the model reasons, sent in the form of its provider; a level the provider does not have is replaced by the nearest one. Without it nothing is sent. `off` is an error on `gemini` and with publisher `google`. |

## `graph.nodes[].validation-node.resources`

| Key | Required | Value | Description |
|---|---|---|---|
| `prompt` | yes | the id of a prompt | The prompt of the judgment: its criterion. Mandatory. |

## `graph.nodes[].validation-node.external`

| Key | Required | Value | Description |
|---|---|---|---|
| `vectordb` | no | list of: map (below) | The vector databases the node searches, as for a node. |
| `search` | no | list of: map (below) | The search engines the node uses. They cannot run yet. |
| `memory` | no | list of: map (below) | The memories the node reads. |

## `graph.nodes[].validation-node.external.vectordb[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a vectordb | The vector database. |
| `query` | yes | text | What is searched: `user_message`, the message of the user, or a fixed text. |
| `utility` | no | map (below) | Utilities on the query and on the chunks. |

## `graph.nodes[].validation-node.external.vectordb[].utility`

| Key | Required | Value | Description |
|---|---|---|---|
| `query` | no | the id of an utility, or list of: the id of an utility | Utilities that change the query, before it is turned into a vector. |
| `chunks` | no | the id of an utility, or list of: the id of an utility | Utilities that change each chunk found, before the prompt. |

## `graph.nodes[].validation-node.external.search[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a search | The search engine. Its result goes where the prompt writes `{{<search-id>}}`. |
| `query` | yes | text | What is searched: `user_message`, the message of the user, or a fixed text. |

## `graph.nodes[].validation-node.external.memory[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a memory | The memory, read whole, both roles, without writing it. |

## `graph.nodes[].validation-node.blackboard`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a blackboard | The blackboard, read without writing it. |

## `graph.nodes[].validation-node.media`

| Key | Required | Value | Description |
|---|---|---|---|
| `input` | no | list of: map (below) | The media the node reads. |

## `graph.nodes[].validation-node.media.input[]`

| Key | Required | Value | Description |
|---|---|---|---|
| `id` | yes | the id of a media | The media. |
| `from` | yes | `resources` or `node` | `resources`: a media with a `source`. `node`: a media produced by a node connected before (`media_not_from_node`); it arrives only if that node passes it on with `node: true`. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities the media goes through before the node uses it. A file on disk reaches a program as its path; everything else reaches a utility as its bytes. They must end with a text or an image. |

## `graph.nodes[].validation-node.message`

| Key | Required | Value | Description |
|---|---|---|---|
| `input` | no | map (below) | Where the text of the node comes from. |

## `graph.nodes[].validation-node.message.input`

| Key | Required | Value | Description |
|---|---|---|---|
| `user_message` | no | `true` or `false` | `true`: the node takes the message of the user, the one given to the command. The default is `false`. |
| `agent_message` | no | `true` or `false` | `true`: the node takes what the node connected before sends on. The default is `false`. |
| `utility` | no | the id of an utility, or list of: the id of an utility | Utilities that change the messages the node takes, which are then used everywhere: in the prompt, in the queries and in what the memories save. A message the node does not take is not changed. |

## `step`

One entry of `graph.edges`.

A map with exactly one of these keys.

| Key | Value | Description |
|---|---|---|
| `start` | the id of a node | The first node: the first entry, and the only `start` (`edge_start`). A graph of one node is its `start` alone. |
| `next` | the id of a node | A node that runs after the entry before it. A node may appear more than once in the edges, and each appearance is a run of it. |
| `end` | the id of a node | The last node: the last entry, and the only `end` (`edge_end`). Every graph needs it, except one of a single node. |
| `linear` | a `group` (below) | A group whose branches run one after the other. `limit` is not allowed. |
| `parallel` | a `group` (below) | A group whose branches run all together, each as a task of its own; with `limit`, at most that many at a time. The first branch that fails ends the run. |
| `concurrent` | a `group` (below) | A group whose branches run all together, at most `limit` at a time; `limit` is mandatory. The first branch that fails ends the run. |
| `batch` | a `group` (below) | A group whose branches are sent to the provider as one batch job; with `limit`, at most that many requests in each job. The branches are plain nodes, all with the same model, without tools, MCP servers or skills. |
| `react` | a `react` (below) | A controller that decides at run time which worker to call. A `react` cannot be a worker of another `react`, nor stand in a group among its workers. |

## `group`

The entries of a group: `limit`, `in`, `fan` and `out`, each written once and in this order.

Each entry is a map with exactly one of these keys.

| Key | Value | Description |
|---|---|---|
| `limit` | whole number | How many branches run at the same time; in `batch`, how many requests go in each job. |
| `in` | the id of a node | The node that runs first. What it sends on is the input of every branch. Mandatory. |
| `fan` | list of: a `branch` (below) | The branches, at least one. Mandatory. |
| `out` | the id of a node | The node that receives the answers of all the branches, packed as one, in the order of the `fan`, and whose output goes on. It may be left out only if no node of the group sends its output on (`group_out_required`). |

## `react`

A controller and the workers it calls.

| Key | Required | Value | Description |
|---|---|---|---|
| `controller` | yes | the id of a node | The node that decides at every turn which worker to call, or gives the final answer, which is the output of the `react`. It has no tools, MCP servers, skills, blackboard or output `schema` of its own. |
| `max_iterations` | yes | whole number | The most calls to workers. One more fails the run (`react_limit`). |
| `components` | yes | list of: a `worker` (below) | The workers the controller may call. Two workers with the same id are an error (`duplicate_id`). |

## `branch`

One branch of a `fan`: a node, or a whole group or `react`.

A map with exactly one of these keys.

| Key | Value | Description |
|---|---|---|
| `next` | the id of a node | A branch made of one node. |
| `linear` | a `group` (below) | A group whose branches run one after the other. `limit` is not allowed. |
| `parallel` | a `group` (below) | A group whose branches run all together, each as a task of its own; with `limit`, at most that many at a time. The first branch that fails ends the run. |
| `concurrent` | a `group` (below) | A group whose branches run all together, at most `limit` at a time; `limit` is mandatory. The first branch that fails ends the run. |
| `batch` | a `group` (below) | A group whose branches are sent to the provider as one batch job; with `limit`, at most that many requests in each job. The branches are plain nodes, all with the same model, without tools, MCP servers or skills. |
| `react` | a `react` (below) | A controller that decides at run time which worker to call. A `react` cannot be a worker of another `react`, nor stand in a group among its workers. |

## `worker`

One worker of a `react`: a node, or a whole group, called by the id of its `in` node.

A map with exactly one of these keys.

| Key | Value | Description |
|---|---|---|
| `worker` | the id of a node | A worker made of one node. |
| `linear` | a `group` (below) | A group whose branches run one after the other. `limit` is not allowed. |
| `parallel` | a `group` (below) | A group whose branches run all together, each as a task of its own; with `limit`, at most that many at a time. The first branch that fails ends the run. |
| `concurrent` | a `group` (below) | A group whose branches run all together, at most `limit` at a time; `limit` is mandatory. The first branch that fails ends the run. |
| `batch` | a `group` (below) | A group whose branches are sent to the provider as one batch job; with `limit`, at most that many requests in each job. The branches are plain nodes, all with the same model, without tools, MCP servers or skills. |
| `react` | a `react` (below) | A controller that decides at run time which worker to call. A `react` cannot be a worker of another `react`, nor stand in a group among its workers. |
