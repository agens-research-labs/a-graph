use super::*;

/// A cut output says where it was cut, and what to do about it.
#[test]
fn cut_says_why() {
    let e = |cut| {
        ModelError::Truncated {
            model: "m".into(),
            provider: "p".into(),
            cut,
        }
        .to_string()
    };
    assert_eq!(
        e(Cut::MaxTokens),
        "model `m` (p): the output was cut before its end, at `max_tokens`: increase `max_tokens`"
    );
    assert_eq!(
        e(Cut::ContextWindow),
        "model `m` (p): the output was cut before its end: the request exceeded the context window of the model"
    );
}
