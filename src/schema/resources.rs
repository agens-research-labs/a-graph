//! `resources`: prompts, tools, MCP servers, media, skills.

use super::key::*;
use super::{Field, Kind, SECRET, Shape};

/// `{ path }` | `{ url, access }` | `{ base64 }`: which one is used, and which are allowed,
/// is decided by the manager of the owner.
const SOURCE_BODY: Shape = Shape::Map(&[
    Field::opt(PATH, Shape::Text).doc("A file, relative to the folder the command is run from."),
    Field::opt(URL, Shape::Text).doc("A URL. It is not read yet: a node that uses it fails when it runs."),
    Field::opt(ACCESS, Shape::Any).doc("How to get in at the `url`, with `url` only. Its content is not defined yet."),
    Field::opt(BASE64, Shape::Text).doc("The content itself, in base64, checked when the graph loads. Only for a media."),
    Field::opt(MIME_TYPE, Shape::Text).doc("The type of the content in `base64`, such as `application/pdf`, given to the utilities the media goes through. Only for a media, with `base64`."),
]);

const PROMPT: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Prompt)).doc("The id of the prompt, cited by the nodes."),
    Field::opt(
        CONTENT,
        Shape::Map(&[
            Field::req(SYSTEM, Shape::Text).doc("The system text, with its placeholders."),
            Field::req(USER, Shape::Text).doc("The user text, with its placeholders."),
        ]),
    )
    .doc("The texts of the prompt, written in the file. A prompt has exactly one of `content` and `source`."),
    Field::opt(SOURCE, SOURCE_BODY).doc(
        "Where the prompt is read from: exactly one of `path`, a `.yml` file with the same `content: { system, user }` (not Markdown, which could not be split into the two texts), and `url`.",
    ),
]);

const TRANSPORT_WORDS: Shape = Shape::Words(&["stdio", "http"]);

const TOOL: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Tool)).doc("The id of the tool: the name the model calls it by, and the id the nodes cite."),
    Field::req(DESCRIPTION, Shape::Text).doc("What the tool does, told to the model."),
    Field::req(TRANSPORT, TRANSPORT_WORDS).doc(
        "`stdio`: the program `command` is started for each call, gets the arguments on its standard input and writes the result on its standard output. `http`: the arguments are POSTed to `url` and the result is the body of the answer.",
    ),
    Field::opt(COMMAND, Shape::Text).doc("The program to start, with `transport: stdio` only (where it is mandatory)."),
    Field::opt(ARGS, Shape::List(&Shape::Text)).doc("The arguments of `command`, with `transport: stdio` only."),
    Field::opt(URL, Shape::Text).doc("The URL the calls are POSTed to, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`."),
    Field::opt(HEADERS, Shape::MapOf(&SECRET)).doc("The headers of the POSTs, with `transport: http` only, each read from an environment variable when the file is checked."),
    Field::req(MAX_ITER, Shape::Int).doc(
        "The most calls of this tool in one run of a node, at least 1, failures included. The count starts again for each node and each appearance of the node in the edges; one more call fails the run (`tool_limit`).",
    ),
    Field::req(TIMEOUT, Shape::Int).doc(
        "The seconds one call may take, at least 1. After that the program is killed or the request abandoned, and the model is told, as for any other failure.",
    ),
    Field::req(PARAMETERS, Shape::Any).doc(
        "The JSON Schema of the arguments, an object, given to the provider as it is written; it must be a valid JSON Schema (`bad_value`). Before a call the arguments the model chose are checked against it: if they do not match, the tool is not called and the model is told why, as for any other failure, and the call counts in `max_iter`.",
    ),
]);

const MCP_SERVER: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Mcp)).doc("The id of the server, cited by the nodes."),
    Field::req(TRANSPORT, TRANSPORT_WORDS).doc("`stdio`: the program `command` is started when the run starts. `http`: the server at `url`, reached with Streamable HTTP."),
    Field::opt(COMMAND, Shape::Text).doc("The program to start, with `transport: stdio` only (where it is mandatory)."),
    Field::opt(ARGS, Shape::List(&Shape::Text)).doc("The arguments of `command`, with `transport: stdio` only."),
    Field::opt(ENV, Shape::MapOf(&SECRET)).doc("The environment variables of the program, with `transport: stdio` only, each read from an environment variable when the server is started."),
    Field::opt(URL, Shape::Text).doc("The URL of the server, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`."),
    Field::opt(HEADERS, Shape::MapOf(&SECRET)).doc("The headers of the requests, with `transport: http` only, each read from an environment variable when the server is connected."),
    Field::req(MAX_ITER, Shape::Int).doc("The most calls to the tools of the server, any of them, in one run of a node, at least 1. One more call fails the run (`tool_limit`)."),
    Field::req(TIMEOUT, Shape::Int).doc("The seconds one call may take, at least 1. After that the model is told that the call failed."),
]);

/// `md`, `file` (a file of any format, made by a utility), or `{ img: <format> }`, `{ audio: <format> }`, `{ video: <format> }`.
const MEDIA_TYPE: Shape = Shape::Either(&[
    Shape::Words(&["md", "file"]),
    Shape::Map(&[Field::req(IMG, Shape::Text).doc("The format of the image: `png`, `jpeg`, `gif`, `webp`...")]),
    Shape::Map(&[Field::req(AUDIO, Shape::Text).doc("The format of the audio: `wav`, `mp3`...")]),
    Shape::Map(&[Field::req(VIDEO, Shape::Text).doc("The format of the video: `mp4`...")]),
]);

const GENERATE_BODY: Shape = Shape::Map(&[
    Field::opt(VOICE, Shape::Text).doc("The voice of an audio. Mandatory for an audio on OpenAI."),
    Field::opt(LANGUAGE_CODE, Shape::Text).doc("The language of an audio. No provider accepts it at present."),
    Field::opt(
        SIZE,
        Shape::Map(&[Field::req(WIDTH, Shape::Int).doc("The width, in pixels."), Field::req(HEIGHT, Shape::Int).doc("The height, in pixels.")]),
    )
    .doc("The size of an image. The provider may accept only some pairs."),
    Field::opt(QUALITY, Shape::Text).doc("The quality of an image. The values depend on the model."),
    Field::opt(BACKGROUND, Shape::Bool).doc(
        "`true` a transparent background, `false` an opaque one; without it the provider decides. A transparent background needs an image of type `png` or `webp` (`media_type_unsupported`).",
    ),
]);

const MEDIA_PROVIDER: Shape = Shape::Map(&[
    Field::opt(
        OPENAI,
        Shape::Map(&[
            Field::opt(MODERATION, Shape::Words(&["low", "auto"])).doc("How strict the moderation of the image is."),
            Field::opt(OUTPUT_COMPRESSION, Shape::Int).doc("From 0 to 100, for an image of type `jpeg` or `webp`."),
            Field::opt(STYLE, Shape::Words(&["vivid", "natural"])).doc(
                "The style of the image. No model accepts it at present: the GPT image models take the style from the \
                 prompt. Not together with the other two (`media_provider_conflict`).",
            ),
        ]),
    )
    .doc("For a model of OpenAI, also through `azure_foundry` with publisher `openai`."),
    Field::opt(
        GEMINI,
        Shape::Map(&[
            Field::opt(IMAGE_SIZE, Shape::Words(&["512", "1K", "2K", "4K"])).doc("The size of the image."),
            Field::opt(ASPECT_RATIO, Shape::Text).doc("The ratio of width to height, for example `16:9`."),
        ]),
    )
    .doc("For a model of Gemini, also through `vertex_ai` with publisher `google`."),
]);

const MEDIA_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Media)).doc("The id of the media, cited by the nodes that read or produce it."),
    Field::req(TYPE, MEDIA_TYPE).doc(
        "What the media is: `md` a text (a file of another format, such as a PDF, is declared `md` and read through a utility that changes it into text); `file` a file of any format made by a utility, delivered in base64; or an image, an audio or a video with its format.",
    ),
    Field::opt(SOURCE, SOURCE_BODY).doc("Where a media that is read comes from: exactly one of `path`, `url` and `base64`. A media has exactly one of `source` and `generate`, except one of type `file`, which has neither."),
    Field::opt(GENERATE, GENERATE_BODY).doc(
        "How a model makes the media. Every value is given to the provider as it is written; a value it does not accept is an error when the file is checked (`not_supported`). A media is one image or one audio, written by one node only (`media_two_writers`).",
    ),
    Field::opt(PROVIDER, MEDIA_PROVIDER).doc("Settings of one family of providers only (`exactly_one`): the family of the model of the node that writes the media."),
]);

const SKILL: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Skill)).doc("The id of the skill, cited by the nodes."),
    Field::req(PATH, Shape::Text).doc("The folder of the skill, with its `SKILL.md`. Its name and description are written in `SKILL.md`, not here."),
    Field::opt(RUN_SCRIPTS, Shape::Bool).doc("Whether the model may run the scripts in the folder `scripts/` of the skill. The default is `true`."),
    Field::opt(MAX_ITER, Shape::Int).doc(
        "The most reads of files and runs of scripts together, for this skill, in one run of a node. The default is 20, at least 1; the activation is not counted. One more fails the run (`tool_limit`).",
    ),
    Field::opt(TIMEOUT, Shape::Int).doc("The seconds a script may take. The default is 30, at least 1."),
    Field::opt(MAX_FILES, Shape::Int).doc("The most files listed when the skill is activated. The default is 200, at least 1."),
    Field::opt(MAX_FILE_BYTES, Shape::Int).doc("The largest file the model may read, and the most bytes kept of the output of a script. The default is 100000, at least 1."),
    Field::opt(MAX_DEPTH, Shape::Int).doc("The most levels of sub-folders listed when the skill is activated. The default is 6, at least 1."),
]);

pub(super) const RESOURCES: Shape = Shape::Map(&[
    Field::opt(PROMPTS, Shape::List(&PROMPT)).doc("The texts the nodes send to their models, with their placeholders."),
    Field::opt(TOOLS, Shape::List(&TOOL)).doc("Programs and services of yours that the model may call while a node runs. The model decides whether to call them; the program makes the call."),
    Field::opt(MCP_SERVERS, Shape::List(&MCP_SERVER)).doc(
        "Servers of the Model Context Protocol, connected when the run starts. Only their tools are used, never their resources or prompts. The arguments of a call are checked against the input schema the server gives for the tool, as for `tools`.",
    ),
    Field::opt(MEDIA, Shape::List(&MEDIA_ITEM)).doc("Everything that is not the text of a message: documents, images, audio, read or produced by the nodes."),
    Field::opt(SKILLS, Shape::List(&SKILL)).doc(
        "Folders in the Agent Skills format (https://agentskills.io/specification): instructions for a task, with their files and scripts, that the model reads when it needs them.",
    ),
]);
