//! `external`: services outside the process (vectordb, search, memory).

use super::key::*;
use super::{Field, Kind, SECRET, Shape};

const VECTORDB_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Vectordb)).doc("The id of the vector database, cited by the nodes and by the memories."),
    Field::req(PROVIDER, Shape::Words(&["qdrant", "chromadb"])).doc("The kind of database."),
    Field::opt(URL, Shape::Text).doc("The URL of the database. Mandatory; its host must be in `allowed_hosts`."),
    Field::opt(API_KEY, SECRET).doc(
        "The key of the database, read from an environment variable when it is used, and sent as the header `api-key` (`qdrant`) or `x-chroma-token` (`chromadb`).",
    ),
    Field::opt(COLLECTION, Shape::Text).doc("The collection that is searched. Mandatory."),
    Field::opt(TOP_K, Shape::Int).doc("How many chunks a search gives back. The default is 5."),
    Field::opt(TIMEOUT, Shape::Int).doc("The seconds a search may take, at least 1. The default is 60."),
    Field::req(EMBEDDING, Shape::Ref(Kind::Embedding)).doc("The embedding model that turns the query into a vector."),
    Field::opt(TEXT_FIELD, Shape::Text).doc("The field of the payload that holds the text of a chunk. Mandatory with `qdrant`, not allowed with `chromadb`, which keeps the text in its documents."),
    Field::opt(TENANT, Shape::Text).doc("The tenant, with `chromadb` only. The default is `default_tenant`."),
    Field::opt(DATABASE, Shape::Text).doc("The database, with `chromadb` only. The default is `default_database`."),
]);

const SEARCH_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Search)).doc("The id of the search engine, cited by the nodes."),
    Field::req(PROVIDER, Shape::Text).doc("The kind of search engine."),
    Field::req(URL, Shape::Text).doc("The URL of the search engine. Its host must be in `allowed_hosts`."),
    Field::opt(ACCESS, Shape::Any).doc("How to get in. Its content is not defined yet."),
]);

const SYNT_CONFIG: Shape = Shape::Map(&[
    Field::req(LENGTH, Shape::Int).doc("How many exchanges the record of a node reaches before they are summarized, at least 1."),
    Field::req(MODEL, Shape::Ref(Kind::Model)).doc("The model that writes the summary."),
    Field::opt(PROMPT, Shape::Ref(Kind::Prompt)).doc(
        "The prompt of the summary, which must use both `{{synt}}` (the previous summary) and `{{chat}}` (the exchanges to summarize). Without it the program uses a prompt of its own.",
    ),
    Field::opt(MAX_TOKENS, Shape::Int).doc("The most tokens of the summary. The default is 4000, since a model that reasons spends part of them before it writes."),
]);

const CHAT_CONFIG: Shape = Shape::Map(&[
    Field::req(LENGTH, Shape::Int).doc("How many exchanges become the history of a call, at least 1."),
    Field::opt(EMBEDDING, Shape::Ref(Kind::Embedding)).doc(
        "The embedding model that turns the messages of the user into vectors. With it, the history is the exchanges most similar to the new message, instead of the last ones.",
    ),
]);

const MEMORY_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Memory)).doc("The id of the memory, cited by the nodes."),
    Field::req(PROVIDER, Shape::Words(&["json_file", "vectordb"])).doc("Where the memory is kept: `json_file`, a JSON file; `vectordb`, a vector database, which cannot run yet."),
    Field::opt(PATH, Shape::Text).doc("The JSON file, with `json_file` only (where it is mandatory). It is created at the first write."),
    Field::opt(REF, Shape::Ref(Kind::Vectordb)).doc("The vector database, with `vectordb` only (where it is mandatory)."),
    Field::opt(SYNT, SYNT_CONFIG).doc("The summary, with `json_file` only. Without it the messages are never summarized, and the memory has no placeholder."),
    Field::opt(CHAT, CHAT_CONFIG).doc("The history, with `json_file` only (where it is mandatory)."),
]);

pub(super) const EXTERNAL: Shape = Shape::Map(&[
    Field::opt(VECTORDB, Shape::List(&VECTORDB_ITEM))
        .doc("Vector databases the nodes search: what they find becomes `{{chunks}}` in the prompt."),
    Field::opt(SEARCH, Shape::List(&SEARCH_ITEM)).doc(
        "Search engines that are not vector databases. They cannot run yet: a node that uses one fails when it runs.",
    ),
    Field::opt(MEMORY, Shape::List(&MEMORY_ITEM)).doc("Memories that keep the conversation of a node between runs."),
]);
