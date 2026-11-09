//! Tests of the topic `vectordb`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "chroma_needs_url_collection/chroma_needs_url_collection.rs"]
mod chroma_needs_url_collection;

#[path = "chroma_reads/chroma_reads.rs"]
mod chroma_reads;

#[path = "chroma_refuses_text_field/chroma_refuses_text_field.rs"]
mod chroma_refuses_text_field;

#[path = "chroma_tenant_db/chroma_tenant_db.rs"]
mod chroma_tenant_db;

#[path = "chromadb_builds/chromadb_builds.rs"]
mod chromadb_builds;

#[path = "chunks_no_prompt/chunks_no_prompt.rs"]
mod chunks_no_prompt;

#[path = "chunks_unbound/chunks_unbound.rs"]
mod chunks_unbound;

#[path = "chunks_unused/chunks_unused.rs"]
mod chunks_unused;

#[path = "database_error/database_error.rs"]
mod database_error;

#[path = "live_chroma/live_chroma.rs"]
mod live_chroma;

#[path = "missing_embedding/missing_embedding.rs"]
mod missing_embedding;

#[path = "qdrant_needs_text_field/qdrant_needs_text_field.rs"]
mod qdrant_needs_text_field;

#[path = "qdrant_no_text/qdrant_no_text.rs"]
mod qdrant_no_text;

#[path = "qdrant_reads/qdrant_reads.rs"]
mod qdrant_reads;

#[path = "qdrant_refuses_tenant/qdrant_refuses_tenant.rs"]
mod qdrant_refuses_tenant;

#[path = "qdrant_rules/qdrant_rules.rs"]
mod qdrant_rules;

#[path = "search_timeout/search_timeout.rs"]
mod search_timeout;

#[path = "static_query/static_query.rs"]
mod static_query;

#[path = "timeout_zero/timeout_zero.rs"]
mod timeout_zero;

#[path = "two_vectordbs_in_order/two_vectordbs_in_order.rs"]
mod two_vectordbs_in_order;

#[path = "unknown_embedding/unknown_embedding.rs"]
mod unknown_embedding;

#[path = "embedding_error/embedding_error.rs"]
mod embedding_error;

#[path = "database_text_kept_out/database_text_kept_out.rs"]
mod database_text_kept_out;
