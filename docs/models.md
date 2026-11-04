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
| `anthropic` | the Anthropic Messages API | `api.anthropic.com` |
| `openai` | the OpenAI Responses API | `api.openai.com` |
| `gemini` | the Gemini API, with an API key (not Vertex AI) | `generativelanguage.googleapis.com` |
| `mistral` | the Mistral API (`api.mistral.ai`). Mistral models hosted on Bedrock, Vertex AI or Azure are not covered | `api.mistral.ai` |
| `bedrock` | Amazon Bedrock, the Converse API | `bedrock-runtime.<region>.amazonaws.com` |
| `vertex_ai` | Google Cloud Vertex AI; `publisher` is `anthropic` or `google` | `aiplatform.googleapis.com` for `location: global`, `aiplatform.<us\|eu>.rep.googleapis.com` for `us` and `eu`, `<location>-aiplatform.googleapis.com` for a region |
| `azure_foundry` | Microsoft Foundry; `publisher` is `anthropic` or `openai` | `<resource>.services.ai.azure.com` for anthropic, `<resource>.openai.azure.com` for openai |
| `ollama` | an [Ollama](https://ollama.com) server | the host of `host` |
| `custom` | a model server of your own that speaks the Ollama contract (below) | the host of `host` |

On `vertex_ai` and `azure_foundry`, `publisher` decides the format of the requests and which keys are required. On
`azure_foundry`, `model` is the name of the deployment created in the Foundry portal.

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

## What each provider can do

What a node asks of its model is checked when the file is checked, before anything runs: a model asked for something
it cannot do is an error (`not_supported`).

| `provider` | Media in | Media out | Batch |
|---|---|---|---|
| `anthropic` | text, images (jpeg, png, gif, webp) | none | yes |
| `openai` | text, images | one image or one audio per call | yes |
| `gemini` | text, images | one image or one audio per call | no |
| `mistral` | text, images | one audio per call (speech) | yes |
| `bedrock` | text, images (png, jpeg, gif, webp) | none | yes, without tools and without an output `schema` |
| `vertex_ai`, publisher `anthropic` | as `anthropic` | none | no |
| `vertex_ai`, publisher `google` | as `gemini` | as `gemini` | no |
| `azure_foundry`, publisher `anthropic` | as `anthropic` | none | no |
| `azure_foundry`, publisher `openai` | as `openai` | as `openai` | yes |
| `ollama`, `custom` | text, images | none | no |

- Audio and video in are not supported yet by any provider.
- A media of type `file` cannot go to a model: a [utility](utilities.md) has to change it into text first.
- On `bedrock`, and with publisher `anthropic` on `vertex_ai` and `azure_foundry`, an `output` with anything but
  `text` is an error (`output_not_text`).
- A node that uses an `anthropic` model, or publisher `anthropic` on `vertex_ai` or `azure_foundry`, must set
  `max_tokens`.
- `gemini`, and `vertex_ai` with publisher `google`, cannot switch reasoning off: `reasoning: off` is an error.
- With `gemini` (and publisher `google`) an output `schema` and a produced media cannot be asked together.

How a produced media is described is explained in [media](media.md).

## Batch

A `batch` group (see [edges](edges.md)) sends the requests of its branches to the provider as one batch job instead of
one call each. The job is submitted, then polled until it ends.

A job the run lets go of before it ended is cancelled at the provider: when the run is stopped (`run_timeout`, the
service shutting down), when a branch beside the group fails, and when the poll of the job fails. The end of the
program waits for each cancel, 3 seconds at most. A run stopped while its batch is prepared (on `openai` and `bedrock`
the requests are first uploaded as a file) sends nothing more, and no job is created. A job whose creation was on its
way is still created, and cancelled as soon as its id comes, within the same 3 seconds. A cancel that fails is written on the standard error, with the id
of the job: `a-graph: cannot cancel the batch job <id> at the provider: <why>`; a creation that did not answer in time
is written too, without an id, since none came. A process that is killed cancels nothing, nor does Ctrl-C on the
command line, which ends the process at once.

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

## Reasoning

The `reasoning` of a node (`off`, `minimal`, `low`, `medium`, `high`, `max`) is sent in the form each provider uses. A
level the provider does not have is replaced by the nearest one: for example `minimal` becomes `low` on `anthropic` and
`bedrock`, and `max` becomes `high` on `gemini` and `ollama`. Without `reasoning` nothing is sent and the provider does
as it wants.

Apart from `off` on Gemini, the program does not know in advance which models can reason: a model that does not accept
the level asked answers with an error of its provider, during the run.

## Ollama and your own models

`ollama` and `custom` take only `host` and `model`. Both are called with `POST <host>/api/chat`, in the format of the
Ollama API: a model server of your own can be used with `provider: custom` if it answers that request in the same way.
`ollama` is kept apart from `custom` so that it can have options of its own in the future; it has none yet.
