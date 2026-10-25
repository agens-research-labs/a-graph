//! The Markdown of a blackboard: the mandatory header and the text below it.
//!
//! ```text
//! ---
//! <description>
//! what this blackboard's content is
//! <purpose>
//! what the content is for
//! ---
//! ```

use super::Place;

const FENCE: &str = "---";
const DESCRIPTION: &str = "<description>";
const PURPOSE: &str = "<purpose>";

/// Where the header ends: the byte after its closing `---` line, or `None` if the text does
/// not start with a header of this shape (a description and a purpose, both with some text).
pub(super) fn header_end(text: &str) -> Result<usize, String> {
    let mut lines = text.split_inclusive('\n');
    if lines.next().map(str::trim_end) != Some(FENCE) {
        return Err("the first line must be `---`".into());
    }
    let mut offset = text.find('\n').map_or(text.len(), |n| n + 1);
    let (mut description, mut purpose) = (None::<bool>, None::<bool>);
    let mut current: Option<&str> = None;
    for line in lines {
        let t = line.trim_end();
        offset += line.len();
        match t {
            FENCE => {
                return match (description, purpose) {
                    (Some(true), Some(true)) => Ok(offset),
                    (None, _) => Err(format!("the header lacks `{DESCRIPTION}`")),
                    (_, None) => Err(format!("the header lacks `{PURPOSE}`")),
                    (Some(false), _) => Err(format!("`{DESCRIPTION}` has no text")),
                    (_, Some(false)) => Err(format!("`{PURPOSE}` has no text")),
                };
            }
            DESCRIPTION => {
                current = Some(DESCRIPTION);
                description = Some(false);
            }
            PURPOSE => {
                current = Some(PURPOSE);
                purpose = Some(false);
            }
            text if !text.trim().is_empty() => match current {
                Some(DESCRIPTION) => description = Some(true),
                Some(PURPOSE) => purpose = Some(true),
                _ => return Err("text before `<description>` in the header".into()),
            },
            _ => {}
        }
    }
    Err("the header is not closed by a `---` line".into())
}

/// The length of the header, 0 if the text has none (a board that lives in memory).
pub(super) fn header_len(text: &str) -> usize {
    header_end(text).unwrap_or(0)
}

/// `text` added to the board, below the header: at the top of the body or at the bottom.
pub(super) fn write(content: &str, text: &str, place: Place) -> String {
    let at = header_len(content);
    let (header, body) = content.split_at(at);
    let (body, text) = (body.trim(), text.trim());
    let mut out = String::from(header);
    let parts: [&str; 2] = match place {
        Place::Top => [text, body],
        Place::Bottom => [body, text],
    };
    for part in parts.iter().filter(|p| !p.is_empty()) {
        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push_str(if out.ends_with('\n') { "\n" } else { "\n\n" });
        }
        out.push_str(part);
        out.push('\n');
    }
    out
}

/// Everything below the header is removed; the header stays.
pub(super) fn cleared(content: &str) -> String {
    content[..header_len(content)].to_string()
}
