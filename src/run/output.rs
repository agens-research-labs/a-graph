//! The result of a run, written for a person or for another program.

use serde_json::{Value, json};

use super::{Content, OutputItem, RunOutput};

impl OutputItem {
    fn message_json(&self) -> Value {
        match &self.message {
            Some(Content::Text(t)) => json!(t),
            Some(Content::Json(j)) => j.clone(),
            None => Value::Null,
        }
    }
}

impl RunOutput {
    /// `{ "items": [ { "node", "message", "media": [ { "id", "mime_type", "data" } ], "reasoning" } ] }`.
    /// `message` is a text, a JSON value for a node with a schema, or null when the node gave none.
    pub fn to_json(&self) -> Value {
        let items: Vec<Value> = self
            .items
            .iter()
            .map(|i| {
                json!({
                    "node": i.agent_id,
                    "message": i.message_json(),
                    "media": i.media.iter().map(|m| json!({ "id": m.id, "mime_type": m.mime_type, "data": m.data })).collect::<Vec<_>>(),
                    "reasoning": i.reasoning,
                })
            })
            .collect();
        json!({ "items": items })
    }

    /// One `node: text` for each item that has a message, in order; only the text, no media, no
    /// reasoning. A text on several lines is written as it is, a JSON value in one line.
    pub fn to_chat_text(&self) -> String {
        self.items
            .iter()
            .filter(|i| i.message.is_some())
            .map(|i| {
                let text = match &i.message {
                    Some(Content::Text(t)) => t.clone(),
                    Some(Content::Json(j)) => j.to_string(),
                    None => String::new(),
                };
                format!("{}: {text}\n", i.agent_id)
            })
            .collect()
    }
}
