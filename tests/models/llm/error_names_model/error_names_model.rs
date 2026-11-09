use super::*;

/// Every error says which model and provider; what the provider wrote is never in the message.
#[test]
fn error_names_model() {
    let e = |err: ModelError| err.to_string();
    let m = || ("m".to_string(), "p".to_string());
    let (model, provider) = m();
    let with_status = e(ModelError::Provider {
        model,
        provider,
        status: Some(429),
        code: Some("rate_limit_error".into()),
        message: String::new(),
        said: Some("slow down".into()),
    });
    assert_eq!(
        with_status,
        "model `m` (p): the provider answered 429 (rate_limit_error)"
    );
    let (model, provider) = m();
    let no_status = e(ModelError::Provider {
        model,
        provider,
        status: None,
        code: None,
        message: "batch expired".into(),
        said: None,
    });
    assert_eq!(
        no_status,
        "model `m` (p): the provider answered with an error: batch expired"
    );
    let (model, provider) = m();
    assert!(e(ModelError::Truncated { model, provider }).contains("`m` (p)"));
    let (model, provider) = m();
    assert!(
        e(ModelError::MimeMismatch {
            model,
            provider,
            expected: "image/png".into(),
            produced: "image/jpeg".into()
        })
        .contains("image/png")
    );
}
