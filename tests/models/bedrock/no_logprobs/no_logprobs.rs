use super::*;

/// The Converse API gives no logprobs: asking them is refused.
#[tokio::test]
async fn no_logprobs() {
    let m = client("http://unused");
    let req = ModelRequirements {
        logprobs: Some(0),
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else {
        panic!("not Unsupported")
    };
    assert_eq!(
        problems,
        vec!["the Converse API of Bedrock gives no logprobs".to_string()]
    );
}
