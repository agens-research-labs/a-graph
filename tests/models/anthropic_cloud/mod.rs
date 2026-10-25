//! Claude on Vertex AI and on Azure Foundry: same format, other URL, other credentials.

use std::sync::{Arc, Mutex};
use serde_json::{Value, json};
use a_graph::models::*;
use a_graph::{Path, Rule};
use crate::common::http::{Req, Resp, serve};

fn vertex_model(location: &str) -> VertexModel {
    VertexModel {
        common: Common { id: "claude_gcp".into(), model: "claude-sonnet-5-5".into(), input: vec![Modality::Text], output: vec![Modality::Text] },
        publisher: VertexPublisher::Anthropic,
        project: "my-project".into(),
        location: location.into(),
        batch: None,
    }
}

fn azure_model(credentials: AzureCredentials) -> AzureModel {
    AzureModel {
        common: Common { id: "claude_azure".into(), model: "my-claude-deployment".into(), input: vec![Modality::Text], output: vec![Modality::Text] },
        publisher: AzurePublisher::Anthropic,
        resource: "My-Res".into(),
        credentials,
        batch: None,
    }
}

fn call(user: &str) -> ModelInvocation {
    ModelInvocation {
        system: String::new(),
        history: vec![],
        user: user.into(),
        media: vec![],
        tools: vec![],
        output_media: vec![],
        schema: None,
        temperature: None,
        top_p: None,
        max_tokens: Some(100),
        reasoning: None,
        tool_rounds: vec![],
    }
}

fn answer() -> String {
    json!({ "id": "msg_1", "type": "message", "role": "assistant", "model": "claude-sonnet-5-5", "content": [{ "type": "text", "text": "hello" }],
            "stop_reason": "end_turn", "usage": { "input_tokens": 12, "output_tokens": 7 } })
    .to_string()
}

async fn serve_once() -> (String, Arc<Mutex<Vec<Req>>>) {
    serve(|_| Resp::ok(answer())).await
}

struct NoToken;

#[async_trait::async_trait]
impl TokenSource for NoToken {
    async fn token(&self) -> Result<String, String> {
        Err("no Google credentials found".into())
    }
}

#[path = "urls/urls.rs"]
mod urls;

#[path = "vertex_body/vertex_body.rs"]
mod vertex_body;

#[path = "azure_deployment/azure_deployment.rs"]
mod azure_deployment;

#[path = "azure_ambient/azure_ambient.rs"]
mod azure_ambient;

#[path = "ambient_missing/ambient_missing.rs"]
mod ambient_missing;

#[path = "no_batch/no_batch.rs"]
mod no_batch;

#[path = "azure_key_env/azure_key_env.rs"]
mod azure_key_env;
