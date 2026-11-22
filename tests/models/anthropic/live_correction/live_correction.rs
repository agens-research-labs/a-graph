use super::*;

/// Live: the API accepts a correction of `schema_prompt` after a tool round.
#[tokio::test]
#[ignore = "calls the real API with CLAUDE_API_KEY"]
async fn live_correction() {
    let model = live_model();
    crate::common::live_correction::correction_accepted(&model, call("")).await;
}
