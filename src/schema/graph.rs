//! `graph`: the nodes and the edges.

use super::key::*;
use super::{Field, Kind, Shape};

const NODE_MODEL: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Model)).doc("The model the node calls."),
    Field::opt(TEMPERATURE, Shape::Number).doc("Given to the provider as it is written."),
    Field::opt(TOP_P, Shape::Number).doc("Given to the provider as it is written."),
    Field::opt(MAX_TOKENS, Shape::Int).doc(
        "The most tokens of the answer: the cap on the cost, set for each node. Mandatory with `anthropic`, and with publisher `anthropic` on `vertex_ai` and `azure_foundry`; without it the provider decides.",
    ),
    Field::opt(REASONING, Shape::Words(&["off", "minimal", "low", "medium", "high", "max"])).doc(
        "How much the model reasons, sent in the form of its provider; a level the provider does not have is replaced by the nearest one. Without it nothing is sent. `off` is an error on `gemini` and with publisher `google`.",
    ),
]);

/// The utilities of a vectordb of a node: on the question before it goes, on the chunks before the prompt.
const VECTORDB_UTILITY: Shape = Shape::Map(&[
    Field::opt(QUERY, UTILITY_LIST).doc("Utilities that change the query, before it is turned into a vector."),
    Field::opt(CHUNKS, UTILITY_LIST).doc("Utilities that change each chunk found, before the prompt."),
]);

const QUERY_FIELD: Field =
    Field::req(QUERY, Shape::Text).doc("What is searched: `user_message`, the message of the user, or a fixed text.");

const QUERY_REF_VECTORDB: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Vectordb)).doc("The vector database."),
    QUERY_FIELD,
    Field::opt(UTILITY, VECTORDB_UTILITY).doc("Utilities on the query and on the chunks."),
]);
const QUERY_REF_SEARCH: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Search))
        .doc("The search engine. Its result goes where the prompt writes `{{<search-id>}}`."),
    QUERY_FIELD,
]);

const ROLES: Shape = Shape::Map(&[
    Field::req(USER, Shape::Bool).doc("The messages of the user."),
    Field::req(ASSISTANT, Shape::Bool).doc("The answers of the node."),
    Field::opt(UTILITY, UTILITY_LIST).doc("Utilities that change, in `read`, each message read back and the summary; in `write`, what is saved, after the utilities of `message.output`."),
]);

const NODE_MEMORY_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Memory))
        .doc("The memory. Its summary, if it has `synt`, goes where the prompt writes `{{<memory-id>}}`."),
    Field::req(READ, ROLES).doc("Which roles of the saved conversation come back as the history of the call."),
    Field::req(WRITE, ROLES).doc("Which roles of this exchange are saved."),
]);

const NODE_EXTERNAL: Shape = Shape::Map(&[
    Field::opt(VECTORDB, Shape::List(&QUERY_REF_VECTORDB)).doc("The vector databases the node searches, in this order: what they find goes where the prompt writes `{{chunks}}`."),
    Field::opt(SEARCH, Shape::List(&QUERY_REF_SEARCH)).doc("The search engines the node uses. They cannot run yet."),
    Field::opt(MEMORY, Shape::List(&NODE_MEMORY_ITEM)).doc("The memories the node reads and writes."),
]);

const NODE_RESOURCES: Shape = Shape::Map(&[
    Field::opt(PROMPT, Shape::Ref(Kind::Prompt)).doc(
        "The prompt of the node. Without it the model gets no system text and its user text is the message of the node, which needs exactly one of the two messages (`prompt_required`).",
    ),
    Field::opt(TOOLS, Shape::List(&Shape::Ref(Kind::Tool))).doc("The tools the model may call."),
    Field::opt(MCP_SERVERS, Shape::List(&Shape::Ref(Kind::Mcp))).doc("The MCP servers whose tools the model may call."),
    Field::opt(SKILLS, Shape::List(&Shape::Ref(Kind::Skill))).doc("The skills the model may use."),
    Field::opt(MAX_ROUNDS, Shape::Int).doc(
        "The most rounds in which the model may ask for tools in one run of the node. The default is 20, at least 1. Every round counts, whatever it asks for: also a skill, or a tool the node does not have, which no `max_iter` counts. One more round fails the run (`tool_limit`).",
    ),
]);

const NODE_BLACKBOARD: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Blackboard)).doc("The blackboard."),
    Field::opt(READ, Shape::Bool).doc("`true`: its text goes where the prompt writes `{{<blackboard-id>}}`, which is then mandatory. The default is `false`."),
    Field::opt(
        WRITE,
        Shape::Map(&[
            Field::req(ENABLED, Shape::Bool).doc("Whether the node writes."),
            Field::req(PLACE, Shape::Words(&["top", "bottom"])).doc("Where the text is added: at the top or at the bottom of what is already below the header."),
            Field::opt(UTILITY, UTILITY_LIST).doc("Utilities that change the text written, after the utilities of `message.output`."),
        ]),
    )
    .doc("After the node answers, its text is added to the blackboard, separated by an empty line."),
    Field::opt(CLEANUP, Shape::Bool).doc("`true`: when the node is done, everything below the header is removed. Not with an enabled `write` (`cleanup_write`). The default is `false`."),
]);

const MEDIA_IN: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Media)).doc("The media."),
    Field::req(FROM, Shape::Words(&["resources", "node"])).doc(
        "`resources`: a media with a `source`. `node`: a media produced by a node connected before (`media_not_from_node`); it arrives only if that node passes it on with `node: true`.",
    ),
    Field::opt(UTILITY, UTILITY_LIST).doc("Utilities the media goes through before the node uses it. A file on disk reaches a program as its path; everything else reaches a utility as its bytes. They must end with a text or an image."),
]);

const MEDIA_OUT: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Media)).doc("The media."),
    Field::opt(FROM, Shape::Words(&["model", "message"])).doc(
        "`model` (the default): the model makes it, so the media must have `generate`. `message`: a utility makes it from the text of the node, so the media must be of type `file` and `utility` is mandatory.",
    ),
    Field::opt(UTILITY, UTILITY_LIST).doc("Utilities that change the file the model made, or that make the file from the text of the node. They must end with a file."),
    Field::opt(ASSISTANT, Shape::Bool).doc("`true`: the media goes to the final output. The default is `false`."),
    Field::opt(NODE, Shape::Bool).doc("`true`: the media goes to the node connected after. The default is `false`."),
]);

const MEDIA_INPUT_FIELD: Field = Field::opt(INPUT, Shape::List(&MEDIA_IN)).doc("The media the node reads.");

const NODE_MEDIA: Shape = Shape::Map(&[
    MEDIA_INPUT_FIELD,
    Field::opt(OUTPUT, Shape::List(&MEDIA_OUT)).doc("The media the node produces."),
]);

/// The utilities of a place, in the order they run: a list of ids, or one id alone.
const UTILITY_LIST: Shape = Shape::Either(&[Shape::Ref(Kind::Utility), Shape::List(&Shape::Ref(Kind::Utility))]);

const MESSAGE_INPUT: Shape = Shape::Map(&[
    Field::opt(USER_MESSAGE, Shape::Bool).doc("`true`: the node takes the message of the user, the one given to the command. The default is `false`."),
    Field::opt(AGENT_MESSAGE, Shape::Bool).doc("`true`: the node takes what the node connected before sends on. The default is `false`."),
    Field::opt(UTILITY, UTILITY_LIST).doc("Utilities that change the messages the node takes, which are then used everywhere: in the prompt, in the queries and in what the memories save. A message the node does not take is not changed."),
]);

const MESSAGE_INPUT_FIELD: Field = Field::opt(INPUT, MESSAGE_INPUT).doc("Where the text of the node comes from.");

const NODE_MESSAGE: Shape = Shape::Map(&[
    MESSAGE_INPUT_FIELD,
    Field::opt(
        OUTPUT,
        Shape::Map(&[
            Field::opt(ASSISTANT_MESSAGE, Shape::Bool).doc("`true`: the answer goes to the final output. The default is `false`."),
            Field::opt(NODE, Shape::Bool).doc("`true`: the answer goes to the node connected after, as its `agent_message`. The default is `false`."),
            Field::opt(REASONING, Shape::Bool).doc("`true`: the reasoning of the model goes to the final output, even when `assistant_message` is false. It never goes to another node. The default is `false`."),
            Field::opt(LOGPROBS, Shape::Int).doc("From 0 to 20: the log probability of every token of the answer goes to the final output, even when `assistant_message` is false, with that many most likely alternatives for each token; on `ollama` and `custom` a model that reasons gives those of its reasoning too, before them. It never goes to another node. Only on `openai`, `mistral`, `ollama`, `custom` and `azure_foundry` with publisher `openai`. Without it nothing is asked."),
            Field::opt(SCHEMA, Shape::Any).doc("A JSON Schema, written as JSON in a text or as YAML: the model answers in that shape, and the answer is a JSON value. It must be a valid JSON Schema (`bad_value`)."),
            Field::opt(UTILITY, UTILITY_LIST).doc("Utilities that change the answer once, before every destination."),
        ]),
    )
    .doc("Where the answer of the node goes."),
]);

// A validation node reads and judges, it produces nothing: it has the blocks of a node where a
// node has them, but only what it reads. `read` is implicit, and tools, MCP servers, skills,
// writing and every output are not accepted.
const JUDGE_RESOURCES: Shape = Shape::Map(&[
    Field::req(PROMPT, Shape::Ref(Kind::Prompt)).doc("The prompt of the judgment: its criterion. Mandatory.")
]);
const JUDGE_MEMORY_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Ref(Kind::Memory)).doc("The memory, read whole, both roles, without writing it.")
]);
const JUDGE_EXTERNAL: Shape = Shape::Map(&[
    Field::opt(VECTORDB, Shape::List(&QUERY_REF_VECTORDB))
        .doc("The vector databases the node searches, as for a node."),
    Field::opt(SEARCH, Shape::List(&QUERY_REF_SEARCH)).doc("The search engines the node uses. They cannot run yet."),
    Field::opt(MEMORY, Shape::List(&JUDGE_MEMORY_ITEM)).doc("The memories the node reads."),
]);
const JUDGE_BLACKBOARD: Shape =
    Shape::Map(&[Field::req(ID, Shape::Ref(Kind::Blackboard)).doc("The blackboard, read without writing it.")]);
const JUDGE_MEDIA: Shape = Shape::Map(&[MEDIA_INPUT_FIELD]);
const JUDGE_MESSAGE: Shape = Shape::Map(&[MESSAGE_INPUT_FIELD]);

const NODE_MODEL_DOC: &str = "The model of the node and the settings of its calls.";

const VALIDATION_NODE_BODY: Shape = Shape::Map(&[
    Field::req(MODEL, NODE_MODEL).doc(NODE_MODEL_DOC),
    Field::req(RESOURCES, JUDGE_RESOURCES).doc("The prompt of the node, its only resource."),
    Field::opt(EXTERNAL, JUDGE_EXTERNAL).doc("The vector databases, search engines and memories the node reads."),
    Field::opt(BLACKBOARD, JUDGE_BLACKBOARD).doc("The blackboard the node reads."),
    Field::opt(MEDIA, JUDGE_MEDIA).doc("The media the node reads."),
    Field::opt(MESSAGE, JUDGE_MESSAGE).doc("Where the text the node judges comes from."),
    Field::opt(RESPONSE, Shape::Any).doc("Accepted and not used: the answer of a validation node always has the shape `{ validation, reasons, log }`."),
    Field::req(ON_ERROR, Shape::Words(&["stop", "node"])).doc(
        "What a refusal does. `stop`: the run ends, and the final output gets the reasons and the log. `node`: the run goes on, and the next node receives the problem as its `agent_message`; it needs a node after it.",
    ),
]);

const NODE_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Node)).doc("The id of the node, cited by the edges."),
    Field::opt(MODEL, NODE_MODEL).doc("The model of the node and the settings of its calls. Mandatory, except on a validation node, which has it inside `validation-node`."),
    Field::opt(EXTERNAL, NODE_EXTERNAL).doc("The vector databases, search engines and memories the node uses."),
    Field::opt(RESOURCES, NODE_RESOURCES).doc("The prompt, the tools, the MCP servers and the skills of the node."),
    Field::opt(BLACKBOARD, NODE_BLACKBOARD).doc("The one blackboard the node manages."),
    Field::opt(MEDIA, NODE_MEDIA).doc("The media the node reads and produces."),
    Field::opt(MESSAGE, NODE_MESSAGE).doc("Where the text of the node comes from and where its answer goes."),
    Field::opt(VALIDATION_NODE, VALIDATION_NODE_BODY).doc(
        "Makes the node a validation node, which reads and judges and produces nothing; the node then has only `id` and `validation-node`. Approved, the message it examined goes on unchanged. It stands in the main flow only, never in a group or a `react`.",
    ),
]);

pub(super) const GRAPH: Shape = Shape::Map(&[
    Field::req(NODES, Shape::List(&NODE_ITEM))
        .doc("The nodes: each a model, what it uses, where its input comes from and where its output goes."),
    Field::req(EDGES, super::edges::EDGES).doc("The order in which the nodes run: a list read in order."),
]);
