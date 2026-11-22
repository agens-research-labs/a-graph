use super::*;

/// Live: the API accepts a correction of `schema_prompt` after a tool round.
#[tokio::test]
#[ignore = "calls the real API with OPENAI_API_KEY"]
async fn live_correction() {
    let model = OpenAi::new(&api_model("OPENAI_API_KEY"), &Path::root()).unwrap();
    crate::common::live_correction::correction_accepted(&model, call("")).await;
}
