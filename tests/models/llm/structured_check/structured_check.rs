use super::*;
use a_graph::models::structured::{Checked, check};
use serde_json::json;

/// What an answer to a schema is: the stop reason first, then the JSON taken out of the text, then the schema.
#[test]
fn structured_check() {
    let schema = json!({ "type": "object", "properties": { "a": { "type": "integer" } }, "required": ["a"] });
    let answer = |text: &str, stop: StopReason| {
        let mut r = Fake { id: "m" }.response(text.to_string());
        r.stop_reason = stop;
        check(&schema, &r)
    };

    assert_eq!(answer("{\"a\": 1}", StopReason::End), Checked::Valid(json!({ "a": 1 })));
    assert_eq!(
        answer("```json\n{\"a\": 1}\n```", StopReason::End),
        Checked::Valid(json!({ "a": 1 }))
    );
    // a refusal and a cut answer are what they are, whatever the text
    assert_eq!(answer("{\"a\": 1}", StopReason::Refused), Checked::Refused);
    assert!(matches!(
        answer("{\"a\":", StopReason::MaxTokens),
        Checked::Incomplete {
            cut: Some(Cut::MaxTokens),
            ..
        }
    ));
    assert!(matches!(
        answer("{\"a\":", StopReason::ContextExceeded),
        Checked::Incomplete {
            cut: Some(Cut::ContextWindow),
            ..
        }
    ));
    // a JSON left open by a model that ended by itself
    assert!(matches!(
        answer("{\"a\": 1", StopReason::End),
        Checked::Incomplete { cut: None, .. }
    ));
    for text in [
        "",
        "six",
        "{\"a\": 1} {\"a\": 2}",
        "Here: {\"a\": 1}",
        "```python\n{\"a\": 1}\n```",
    ] {
        assert!(
            matches!(answer(text, StopReason::End), Checked::NotJson { .. }),
            "{text:?}"
        );
    }
    assert!(matches!(
        answer("{\"a\": \"one\"}", StopReason::End),
        Checked::SchemaMismatch { reasons } if reasons.len() == 1 && reasons[0].contains("/a")
    ));
}
