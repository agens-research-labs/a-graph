use super::*;

/// Check lists every problem and names model and provider.
#[test]
fn check_names_all() {
    let model = Fake { id: "fast" };
    assert!(model.check(&ModelRequirements::default()).is_ok());

    let req = ModelRequirements {
        batch: true,
        uses_tools: true,
        schema: Some(serde_json::json!({})),
        ..Default::default()
    };
    let err = model.check(&req).unwrap_err();
    let text = err.to_string();
    assert!(text.contains("`fast`") && text.contains("(fake)"), "{text}");
    assert!(
        text.contains("batch is not supported") && text.contains("tools and schema together"),
        "{text}"
    );
    assert!(matches!(err, ModelError::Unsupported { ref problems, .. } if problems.len() == 2));
}
