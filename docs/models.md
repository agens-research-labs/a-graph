# Models

The `models` block lists the language models of the graph. Each has an `id`, and a node uses one by citing it. The key
`provider` chooses who serves the model, and with it which other keys the entry has. Every key is described in
[the reference](graph.md); this page explains what the providers do.

```yaml
models:
  - id: writer
    provider: anthropic
    model: claude-sonnet-5-5
    input: [text, image]
    output: [text]
    api_key: { env: ANTHROPIC_API_KEY }
```

## Providers

| `provider` | Service | Host checked against `allowed_hosts` |
|---|---|---|
| `anthropic` | the Anthropic Messages API | `api.anthropic.com`, or the host of `host` (see [a gateway](#a-gateway-in-front-of-a-provider)) |
| `openai` | the OpenAI Responses API | `api.openai.com`, or the host of `host` |
| `gemini` | the Gemini API, with an API key (not Vertex AI) | `generativelanguage.googleapis.com`, or the host of `host` |
| `mistral` | the Mistral API (`api.mistral.ai`). Mistral models hosted on Bedrock, Vertex AI or Azure are not covered | `api.mistral.ai`, or the host of `host` |
| `bedrock` | Amazon Bedrock, the Converse API | `bedrock-runtime.<region>.amazonaws.com` |
| `vertex_ai` | Google Cloud Vertex AI; `publisher` is `anthropic` or `google` | `aiplatform.googleapis.com` for `location: global`, `aiplatform.<us\|eu>.rep.googleapis.com` for `us` and `eu`, `<location>-aiplatform.googleapis.com` for a region |
| `azure_foundry` | Microsoft Foundry; `publisher` is `anthropic` or `openai` | `<resource>.services.ai.azure.com` for anthropic, `<resource>.openai.azure.com` for openai |
| `ollama` | an [Ollama](https://ollama.com) server | the host of `host` |
| `custom` | a model server of your own that speaks the Ollama contract (below) | the host of `host` |

On `vertex_ai` and `azure_foundry`, `publisher` decides the format of the requests and which keys are required. On
`azure_foundry`, `model` is the name of the deployment created in the Foundry portal.

## A gateway in front of a provider

`anthropic`, `openai`, `gemini` and `mistral` take an optional `host`: the server the calls go to instead of the API
of the provider, for example an AI gateway that meters the calls, sets quotas or holds the keys of the providers, or
a proxy of the company. The value is the base of the API, as the SDK of the provider takes it, and the paths of the
API go under it:

| `provider` | Default `host` | A call goes to |
|---|---|---|
| `anthropic` | `https://api.anthropic.com` | `<host>/v1/messages` |
| `openai` | `https://api.openai.com/v1` | `<host>/responses` |
| `gemini` | `https://generativelanguage.googleapis.com` | `<host>/v1beta/interactions` |
| `mistral` | `https://api.mistral.ai` | `<host>/v1/chat/completions` |

```yaml
models:
  - id: smart
    provider: openai
    model: gpt-5
    input: [text]
    output: [text]
    api_key: { env: GATEWAY_KEY }
    host: https://gateway.example.com/openai/v1
```

- The host of `host`, on its port, must be in `allowed_hosts`, as for a local server; the host of the provider is no
  longer needed, since nothing goes there.
- Everything goes to `host`, the batches too (`anthropic`, `openai`, `mistral`), and the key named by `api_key`
  goes with them: a gateway that does not serve the batch API of the provider answers with an error, which stops the
  run (`model_failed`).
- The requests are those of the provider, unchanged: the gateway must speak the same API. `openai` uses the Responses
  API, not Chat Completions, so a server that is "compatible with OpenAI" only for Chat Completions does not work.
  The produced media of `openai` and `mistral` (images, speech) go under `host` as well.

## Credentials

- `anthropic`, `openai`, `gemini` and `mistral` take an `api_key`, read from an environment variable when the file is
  checked (see [the file](file.md#secrets)).
- `bedrock` takes `credentials: ambient`, or the keys `access_key`, `secret_key` and, for temporary credentials,
  `session_token`.
- `vertex_ai` takes `credentials: ambient` only: the Google Application Default Credentials.
- `azure_foundry` takes `credentials: ambient`, or an `api_key`. With `ambient` the program tries, in this order, the
  managed identity of an Azure resource, the Azure CLI and the Azure Developer CLI.
- `ollama` and `custom` take no credentials.

`ambient` credentials are resolved by the SDK of the cloud provider at the first call. The traffic needed to resolve
them (metadata servers, token endpoints) is not checked against `allowed_hosts`; the explicit keys contact only the
hosts of the model.

On `bedrock` the hosts are always the standard ones of the region: the FIPS and dual-stack endpoints, and the
endpoints set in the environment or in the AWS profile (`AWS_ENDPOINT_URL` and similar), are not used, so the hosts
called are the hosts checked.

A call to a model that does not end within 10 minutes is abandoned and fails like a network error. On `bedrock` the
same limit applies to each call to Bedrock and to S3, and to the reading of each batch result file. The limit covers
the retries of the AWS SDK too: a call that fails after 9 minutes has one minute left for them. The upload of the
input of a batch is a call too: a file of some hundred MB on a slow link may not fit in it.

## What each provider can do

What a node asks of its model is checked when the file is checked, before anything runs: a model asked for something
it cannot do is an error (`not_supported`).

| `provider` | Media in | Media out | Batch | Logprobs | Tools | Schema | Refusal |
|---|---|---|---|---|---|---|---|
| `anthropic` | text, images (jpeg, png, gif, webp) | none | yes | no | yes | yes | yes |
| `openai` | text, images | one image or one audio per call | yes | yes, for text | yes | yes, not with a produced media | yes |
| `gemini` | text, images | one image or one audio per call | no | no | yes | yes, not with a produced media | no |
| `mistral` | text, images | one audio per call (speech) | yes | yes, for text | yes | yes, not with a produced media | no |
| `bedrock` | text, images (png, jpeg, gif, webp) | none | yes, without an output `schema` | no | yes | yes, not in a batch | yes |
| `vertex_ai`, publisher `anthropic` | as `anthropic` | none | no | no | yes | yes | yes |
| `vertex_ai`, publisher `google` | as `gemini` | as `gemini` | no | no | yes | as `gemini` | no |
| `azure_foundry`, publisher `anthropic` | as `anthropic` | none | no | no | yes | yes | yes |
| `azure_foundry`, publisher `openai` | as `openai` | as `openai` | yes | as `openai` | yes | as `openai` | yes |
| `ollama`, `custom` | text, images | none | no | yes | yes | yes, see below | no |

- Audio and video in are not supported yet by any provider.
- A media of type `file` cannot go to a model: a [utility](utilities.md) has to change it into text first.
- On `bedrock`, and with publisher `anthropic` on `vertex_ai` and `azure_foundry`, an `output` with anything but
  `text` is an error (`output_not_text`).
- A node that uses an `anthropic` model, or publisher `anthropic` on `vertex_ai` or `azure_foundry`, must set
  `max_tokens`.
- `gemini`, and `vertex_ai` with publisher `google`, cannot switch reasoning off: `reasoning: off` is an error.
- With `openai`, `gemini` and `mistral` (and their publishers) an output `schema` and a produced media cannot be asked
  together: the media comes with no text to match the schema (`not_supported`).
- Schema: an output `schema`, and the verdict of a judge or the routing output of a `react` with
  `structured_output: api`, enforced by the API (see [nodes](nodes.md)). On `ollama` the cloud models ignore it: use
  `schema_prompt`, or `structured_output: prompt`. On `custom` whether it is enforced depends on the server.
- Refusal: the provider says that the model refused, and an answer to a schema then stops the run (`model_refused`).
  A provider with no refusal gives a refusal as an ordinary text, which is read as one.

How a produced media is described is explained in [media](media.md).

## Batch

A `batch` group (see [edges](edges.md)) sends the requests of its branches to the provider as one batch job instead of
one call each. The job is submitted, then polled until it ends.

A poll of the job, or the download of its results, that fails with an error that may not come again (the network,
429, 5xx such as the 529 of Anthropic) is tried again at the next poll; the batch fails after 10 such failures in a row,
about 5 minutes. Any other error ends it at once. The request that creates the job is never tried again: one that
arrived but whose answer was lost would create a second job. `bedrock` has the retries of the AWS SDK instead: 3
attempts.

A job the run lets go of before it ended is cancelled at the provider: when the run is stopped (`run_timeout`, the
service shutting down, Ctrl-C on the command line), when a branch beside the group fails, and when the poll of the job
fails. The end of the program waits for each cancel, 3 seconds at most. A run stopped while its batch is prepared (on
`openai` and `bedrock` the requests are first uploaded as a file) sends nothing more, and no job is created. A job whose
creation was on its way is still created, and cancelled as soon as its id comes, within the same 3 seconds. A cancel
the provider refuses is followed by one read of the status of the job: a job that has ended, or that is already being
cancelled, leaves nothing running, and the refusal is not a failure. Any other cancel that fails is written on the
standard error, with the id of the job: `a-graph: cannot cancel the batch job <id> at the provider: <why>`; a creation that did not answer in time is written too, without an id, since none came. A process that
is killed cancels nothing, nor does a second Ctrl-C, which ends the process at once.

- `bedrock` needs a `batch` block on the model: the S3 folder where the requests are written, the S3 folder where the
  answers are written, and the IAM role the job runs with. The batch reaches two more services, and their hosts must
  be in `allowed_hosts` too: the Bedrock control API (`bedrock.<region>.amazonaws.com`), where the job is created and
  followed, and the S3 bucket of each folder (`<bucket>.s3.<region>.amazonaws.com` for a usual bucket name). A folder
  that is not an `s3://` URI is an error of the file (`bad_value`).
- `azure_foundry` with publisher `openai` may have a `batch` block that names its Global-Batch deployment.
- `anthropic`, `openai` and `mistral` need nothing more.
- A `batch` block on `vertex_ai` is accepted only with publisher `google`, and on `azure_foundry` only with publisher
  `openai` (`batch_not_allowed`). With publisher `google` the batch is not available yet.
- A produced media cannot go in a batch.
- The branches of a batch cannot have tools, MCP servers or skills, with any provider (see [edges](edges.md)).

## Reasoning

The `reasoning` of a node (`off`, `minimal`, `low`, `medium`, `high`, `max`) is sent in the form each provider uses. A
level the provider does not have is replaced by the nearest one: for example `minimal` becomes `low` on `anthropic` and
`bedrock`, and `max` becomes `high` on `gemini` and `ollama`. Without `reasoning` nothing is sent and the provider does
as it wants.

Apart from `off` on Gemini, the program does not know in advance which models can reason: a model that does not accept
the level asked answers with an error of its provider, during the run.

## Logprobs

The `logprobs` of a node (see [nodes](nodes.md#where-the-answer-goes)) are asked in the form each provider uses:

- `openai` (and `azure_foundry` with publisher `openai`): `include` gets `message.output_text.logprobs`, and
  `top_logprobs` the number of alternatives;
- `mistral`, `ollama` and `custom`: `logprobs: true`, and `top_logprobs` the number of alternatives.

Only `ollama` was tried with a real model. `openai` and `mistral` were tried only against a test server, and
`azure_foundry` with publisher `openai`, which sends the same request as `openai`, was not tried at all: a provider
may still refuse the request with an error during the run.

On `ollama` and `custom`, a model that reasons gives the tokens of its reasoning too, in the same list, before those of
the answer: with qwen, for example, the reasoning, then `</think>`, then the answer. The program does not separate
them, since the mark between the two changes from model to model: for the tokens of the answer alone the node sets
`reasoning: off`. A model that reasons by default, as qwen does, reasons also when the node says nothing.

With 0 alternatives `top_logprobs` is not sent. In the final output each token is
`{ "token", "logprob", "bytes", "top_logprobs": [ { "token", "logprob", "bytes" } ] }`, the same for every provider;
`bytes` is `null` when the provider does not give them.

- `anthropic`, `bedrock`, `gemini`, `vertex_ai` with publisher `anthropic` or `google`, and `azure_foundry` with
  publisher `anthropic` give no logprobs: a node that asks them is an error (`not_supported`). Gemini has them only
  outside the Interactions API, which is the one the program uses.
- A produced media or speech gives no logprobs: a node that produces one and asks them is an error (`not_supported`).
- In a batch the logprobs are asked as in a single call; a request the provider refuses for them stops the run like
  any refused request of the batch.
- The program does not know in advance which models give them, nor the most alternatives a provider allows beyond
  the 20 of OpenAI: a model that does not accept the request answers with an error of its provider; one that ignores
  it gives an answer without logprobs, which stops the run (`model_failed`).

## Ollama and your own models

`ollama` and `custom` take only `host` and `model`. Both are called with `POST <host>/api/chat`, in the format of the
Ollama API: a model server of your own can be used with `provider: custom` if it answers that request in the same way.
`ollama` is kept apart from `custom` so that it can have options of its own in the future; it has none yet.

## Errors

A provider that answers with an error fails the node (`model_failed`). The message says the status of the answer and,
when the provider gives one, the code of the error: `the provider answered 400 (invalid_request_error)`. The code is
`error.type` for `anthropic`, `error.code` or else `error.type` for `openai`, `error.status` for `gemini`, the code of
the exception for `bedrock`; `mistral`, `ollama` and `custom` give none. A code is kept only if it is made of letters,
digits and `_`, 64 characters at most. The text of the provider, which may repeat what was sent, is the `detail` of
the error, written only as the passport allows (see [observability](observability.md)). The same holds for an
answer that says it failed, for a batch that failed and for a request of a batch that failed, and for the server of
the embeddings.

An answer whose `status` (`openai`, `gemini`) or whose batch result `type` (`anthropic`) is none the program knows
also fails the node. The value follows the same rule as a code: kept in the message only if it is made of letters,
digits and `_`, 64 characters at most; otherwise the message says `an unexpected status` or `an unknown batch result
type` and the value is the `detail`, written only as the passport allows.
