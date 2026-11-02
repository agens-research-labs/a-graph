//! `embeddings`: the models that turn a text into a vector; one list, `provider` chooses the fields.

use super::key::*;
use super::{Field, Kind, Shape, Variant};

const COMMON: &[Field] = &[
    Field::req(ID, Shape::Id(Kind::Embedding))
        .doc("The id of the embedding model, cited by the vector databases and the memories."),
    Field::req(MODEL, Shape::Text).doc("The name of the model at its provider."),
];

const OLLAMA: &[Field] = &[Field::req(HOST, Shape::Text)
    .doc("The URL of the server, for example `http://localhost:11434`. Its host must be in `allowed_hosts`.")];

pub(super) const EMBEDDINGS: Shape = Shape::List(&Shape::Tagged {
    by: PROVIDER,
    by_doc: "The service the model is reached through. It decides the other keys.",
    common: COMMON,
    variants: &[Variant::new(
        "ollama",
        OLLAMA,
        "An Ollama server, called with `POST <host>/api/embed`.",
    )],
});
