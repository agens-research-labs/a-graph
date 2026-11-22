use super::*;

/// Live: the API accepts a correction of `schema_prompt` after a tool round.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY"]
async fn live_correction() {
    let model = live("gemini-3.5-flash-lite");
    crate::common::live_correction::correction_accepted(&model, call("")).await;
}
