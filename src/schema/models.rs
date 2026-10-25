//! `models`: one list; the key `provider` chooses the other fields.

use super::key::*;
use super::{Field, Kind, SECRET, Shape, Variant};

const MODALITIES: Shape = Shape::List(&Shape::Words(&["text", "image", "audio", "video"]));
const AMBIENT: Shape = Shape::Words(&["ambient"]);

const API_KEY_FIELD: Field = Field::req(API_KEY, SECRET).doc("The key of the API, read from an environment variable when the file is checked.");

const COMMON: &[Field] = &[
    Field::req(ID, Shape::Id(Kind::Model)).doc("The id of the model, cited by the nodes."),
    Field::req(MODEL, Shape::Text).doc("The name of the model at its provider; on `azure_foundry`, the name of the deployment created in the Foundry portal."),
    Field::req(INPUT, MODALITIES).doc("What the model reads. The program does not check the nodes against this list: what a node sends is checked against what the provider accepts."),
    Field::req(OUTPUT, MODALITIES).doc(
        "What the model produces. On `bedrock`, and with publisher `anthropic`, anything but `text` is an error (`output_not_text`). Otherwise the program does not check the nodes against this list: what a node asks is checked against what the provider can produce.",
    ),
];

const ANTHROPIC: &[Field] = &[
    API_KEY_FIELD,
    Field::opt(WORKSPACE_ID, Shape::Text).doc("The Anthropic workspace, sent as the header `anthropic-workspace-id`, for a key that is not bound to a workspace. It is an id, not a secret."),
];

const API_KEY_ONLY: &[Field] = &[API_KEY_FIELD];

const BEDROCK: &[Field] = &[
    Field::req(REGION, Shape::Text).doc("The AWS region the model runs in, for example `eu-west-1`."),
    Field::req(
        CREDENTIALS,
        Shape::Either(&[
            AMBIENT,
            Shape::Map(&[
                Field::req(ACCESS_KEY, SECRET).doc("The AWS access key id."),
                Field::req(SECRET_KEY, SECRET).doc("The AWS secret access key."),
                Field::opt(SESSION_TOKEN, SECRET).doc("The session token, only for temporary credentials."),
            ]),
        ]),
    )
    .doc(
        "`ambient`: the default chain of the AWS SDK, resolved at the first call; the traffic it needs is not checked against `allowed_hosts`. Or the keys, each read from an environment variable when the file is checked.",
    ),
    Field::opt(
        BATCH,
        Shape::Map(&[
            Field::req(INPUT_S3_URI, Shape::Text).doc("The S3 folder the requests are written to, as `s3://<bucket>/<prefix>`. Its bucket must be in `allowed_hosts`."),
            Field::req(OUTPUT_S3_URI, Shape::Text).doc("The S3 folder the answers are written to, as `s3://<bucket>/<prefix>`. Its bucket must be in `allowed_hosts`."),
            Field::req(ROLE_ARN, Shape::Text).doc("The IAM role the batch job runs with."),
        ]),
    )
    .doc(
        "Needed only by a `batch` group. The job is created on the Bedrock control API (`bedrock.<region>.amazonaws.com`), which must be in `allowed_hosts`. A batch cannot have tools or an output `schema`.",
    ),
];

const VERTEX_AI: &[Field] = &[
    Field::req(PUBLISHER, Shape::Words(&["anthropic", "google"])).doc("Whose model it is: it decides the format of the requests and what the model can do."),
    Field::req(PROJECT, Shape::Text).doc("The Google Cloud project."),
    Field::req(LOCATION, Shape::Text).doc(
        "Where the model runs: `global` (`aiplatform.googleapis.com`), `us` or `eu` (`aiplatform.<location>.rep.googleapis.com`), or a region (`<location>-aiplatform.googleapis.com`).",
    ),
    Field::req(CREDENTIALS, AMBIENT).doc("The Google Application Default Credentials, resolved at the first call; the traffic they need is not checked against `allowed_hosts`."),
    Field::opt(
        BATCH,
        Shape::Map(&[
            Field::req(INPUT_URI, Shape::Text).doc("Where the requests of the batch are read from."),
            Field::req(OUTPUT_URI_PREFIX, Shape::Text).doc("Where the answers of the batch are written."),
        ]),
    )
    .doc("Accepted only with publisher `google` (`batch_not_allowed`), whose batch is not available yet."),
];

const AZURE_FOUNDRY: &[Field] = &[
    Field::req(PUBLISHER, Shape::Words(&["anthropic", "openai"])).doc("Whose model it is: it decides the format of the requests and what the model can do."),
    Field::req(RESOURCE, Shape::Text).doc("The name of the Foundry resource: the host is `<resource>.services.ai.azure.com` for `anthropic`, `<resource>.openai.azure.com` for `openai`."),
    Field::req(
        CREDENTIALS,
        Shape::Either(&[
            AMBIENT,
            Shape::Map(&[Field::req(API_KEY, SECRET).doc("The key of the resource, read from an environment variable when the file is checked.")]),
        ]),
    )
    .doc(
        "`ambient`: in this order, the managed identity of an Azure resource, the Azure CLI and the Azure Developer CLI, resolved at the first call; the traffic they need is not checked against `allowed_hosts`. Or an `api_key`.",
    ),
    Field::opt(BATCH, Shape::Map(&[Field::req(DEPLOYMENT, Shape::Text).doc("The Global-Batch deployment the batch is sent to.")]))
        .doc("Accepted only with publisher `openai` (`batch_not_allowed`)."),
];

const LOCAL: &[Field] = &[Field::req(HOST, Shape::Text).doc("The URL of the server, for example `http://localhost:11434`. Its host must be in `allowed_hosts`.")];

pub(super) const MODELS: Shape = Shape::List(&Shape::Tagged {
    by: PROVIDER,
    by_doc: "The service the model is reached through. It decides the other keys.",
    common: COMMON,
    variants: &[
        Variant::new("anthropic", ANTHROPIC, "The Anthropic Messages API (`api.anthropic.com`). A node that uses it must set `max_tokens`."),
        Variant::new("openai", API_KEY_ONLY, "The OpenAI Responses API (`api.openai.com`)."),
        Variant::new("gemini", API_KEY_ONLY, "The Gemini API with an API key, not Vertex AI (`generativelanguage.googleapis.com`). It has no batch, and reasoning cannot be switched off."),
        Variant::new("mistral", API_KEY_ONLY, "The Mistral API (`api.mistral.ai`)."),
        Variant::new("bedrock", BEDROCK, "Amazon Bedrock, through the Converse API (`bedrock-runtime.<region>.amazonaws.com`). It produces text only."),
        Variant::new(
            "vertex_ai",
            VERTEX_AI,
            "Google Cloud Vertex AI. With publisher `anthropic` it works as `anthropic` and produces text only; with `google` as `gemini`. Neither has a batch yet.",
        ),
        Variant::new(
            "azure_foundry",
            AZURE_FOUNDRY,
            "Microsoft Foundry. With publisher `anthropic` it works as `anthropic`, produces text only and has no batch; with `openai` as `openai`.",
        ),
        Variant::new("ollama", LOCAL, "An Ollama server."),
        Variant::new("custom", LOCAL, "A model server of your own, called with `POST <host>/api/chat` in the format of the Ollama API."),
    ],
});
