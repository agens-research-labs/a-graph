//! The embedding models of the `embeddings` block: they turn a text into a vector. A vectordb
//! cites one by id to embed the query before searching.

mod manager;
mod ollama;

use async_trait::async_trait;

pub use manager::EmbeddingManager;
pub use ollama::OllamaEmbedding;

use crate::models::ModelError;

/// A model that embeds texts.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// One vector per text, in the same order.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError>;
}
