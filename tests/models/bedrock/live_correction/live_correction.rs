use super::*;

/// Live: the API accepts a correction of `schema_prompt` after a tool round.
#[tokio::test]
#[ignore = "calls the real API with AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY; the model is A_GRAPH_LIVE_BEDROCK_MODEL, anthropic.claude-sonnet-4-5 when unset"]
async fn live_correction() {
    let model = live_bedrock();
    crate::common::live_correction::correction_accepted(&model, call("")).await;
}
