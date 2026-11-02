//! Tests of the topic `embeddings`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "builds/builds.rs"]
mod builds;

#[path = "count_mismatch/count_mismatch.rs"]
mod count_mismatch;

#[path = "embeds_texts/embeds_texts.rs"]
mod embeds_texts;

#[path = "host_refused/host_refused.rs"]
mod host_refused;

#[path = "id_like_model/id_like_model.rs"]
mod id_like_model;

#[path = "live_embeddinggemma/live_embeddinggemma.rs"]
mod live_embeddinggemma;

#[path = "server_error/server_error.rs"]
mod server_error;
