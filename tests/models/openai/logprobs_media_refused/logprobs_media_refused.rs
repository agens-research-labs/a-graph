use super::*;

/// A produced media goes to an API without logprobs: asking them for it is refused; for text they
/// are allowed.
#[tokio::test]
async fn logprobs_media_refused() {
    let m = client("http://unused");
    assert!(
        m.check(&ModelRequirements {
            logprobs: Some(5),
            ..Default::default()
        })
        .is_ok()
    );
    let req = ModelRequirements {
        logprobs: Some(5),
        output_media: vec![image("png", generate(), None)],
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else {
        panic!("not Unsupported")
    };
    assert!(problems.iter().any(|p| p.contains("gives no logprobs")), "{problems:?}");
}
