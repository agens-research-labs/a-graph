//! The file of a `json_file` memory: a list of records, one per node, each with the summary
//! (`synt`) and the current exchanges (`chat`). This module reads it and chooses what a node gets.

use serde::{Deserialize, Serialize};

use super::ChatConfig;

/// One message saved in the file; a user message may carry the vector of its text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// What the file keeps of one node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub synt: String,
    #[serde(default)]
    pub chat: Vec<Entry>,
}

/// A user message with the answer that follows it, if one was saved.
#[derive(Debug, Clone)]
pub struct Exchange {
    pub user: Option<Entry>,
    pub assistant: Option<Entry>,
}

/// Which roles a node takes.
#[derive(Debug, Clone, Copy)]
pub struct Take {
    pub user: bool,
    pub assistant: bool,
}

/// The record of node `node` in the file at `path`. A file that does not exist yet, or has no
/// record of the node, gives an empty record: the file is created at the first write.
pub fn read_record(path: &str, node: &str) -> Result<Record, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Record { id: node.to_string(), ..Record::default() }),
        Err(e) => return Err(format!("cannot read `{path}`: {e}")),
    };
    if text.trim().is_empty() {
        return Ok(Record { id: node.to_string(), ..Record::default() });
    }
    let records: Vec<Record> = serde_json::from_str(&text).map_err(|e| format!("`{path}` is not a list of records: {e}"))?;
    Ok(records.into_iter().find(|r| r.id == node).unwrap_or_else(|| Record { id: node.to_string(), ..Record::default() }))
}

impl Record {
    /// The saved messages grouped in exchanges, in order: each user message opens one.
    pub fn exchanges(&self) -> Vec<Exchange> {
        let mut out: Vec<Exchange> = Vec::new();
        for e in &self.chat {
            match e.role.as_str() {
                "user" => out.push(Exchange { user: Some(e.clone()), assistant: None }),
                _ => match out.last_mut() {
                    Some(last) if last.assistant.is_none() => last.assistant = Some(e.clone()),
                    _ => out.push(Exchange { user: None, assistant: Some(e.clone()) }),
                },
            }
        }
        out
    }

    /// What a node gets of `chat`: the last `length` exchanges, or with an embedding the `length`
    /// exchanges whose user message is most similar to `query`, in the order they were saved.
    /// Only the roles the node takes are kept.
    pub fn select(&self, config: &ChatConfig, query: Option<&[f32]>, take: Take) -> Vec<Entry> {
        let exchanges = self.exchanges();
        let length = config.length as usize;
        let mut chosen: Vec<usize> = match query {
            Some(q) => {
                let mut scored: Vec<(usize, f32)> = exchanges
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| x.user.as_ref().and_then(|u| u.embedding.as_ref()).map(|v| (i, cosine(q, v))))
                    .collect();
                scored.sort_by(|a, b| b.1.total_cmp(&a.1));
                scored.into_iter().take(length).map(|(i, _)| i).collect()
            }
            None => (exchanges.len().saturating_sub(length)..exchanges.len()).collect(),
        };
        chosen.sort_unstable();
        let mut out = Vec::new();
        for i in chosen {
            if let (true, Some(u)) = (take.user, &exchanges[i].user) {
                out.push(u.clone());
            }
            if let (true, Some(a)) = (take.assistant, &exchanges[i].assistant) {
                out.push(a.clone());
            }
        }
        out
    }
}

/// How alike two vectors are, from -1 to 1; 0 when one has no length or their sizes differ.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let (na, nb) = (a.iter().map(|x| x * x).sum::<f32>().sqrt(), b.iter().map(|x| x * x).sum::<f32>().sqrt());
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

/// Writes `records` to `path` in one step (a temporary file, then renamed), creating the folder.
fn write_records(path: &str, records: &[Record]) -> Result<(), String> {
    let target = std::path::Path::new(path);
    if let Some(dir) = target.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
    let temporary = format!("{path}.tmp");
    std::fs::write(&temporary, text).map_err(|e| format!("cannot write `{temporary}`: {e}"))?;
    std::fs::rename(&temporary, target).map_err(|e| format!("cannot replace `{path}`: {e}"))
}

/// Reads the whole list of records; a file that does not exist yet is an empty list.
fn read_records(path: &str) -> Result<Vec<Record>, String> {
    match std::fs::read_to_string(path) {
        Ok(t) if t.trim().is_empty() => Ok(Vec::new()),
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("`{path}` is not a list of records: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("cannot read `{path}`: {e}")),
    }
}

/// Changes the record of `node` (created if it does not exist), sets its `timestamp` and saves
/// the file. Gives back the record as it is after the change.
pub fn update_record(path: &str, node: &str, change: impl FnOnce(&mut Record)) -> Result<Record, String> {
    let mut records = read_records(path)?;
    let index = match records.iter().position(|r| r.id == node) {
        Some(i) => i,
        None => {
            records.push(Record { id: node.to_string(), ..Record::default() });
            records.len() - 1
        }
    };
    change(&mut records[index]);
    records[index].timestamp = chrono::Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
    write_records(path, &records)?;
    Ok(records.swap_remove(index))
}
