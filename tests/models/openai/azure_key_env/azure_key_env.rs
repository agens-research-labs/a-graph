use super::*;

/// The azure key comes from the environment.
#[test]
fn azure_key_env() {
    let creds = AzureCredentials::ApiKey("A_GRAPH_TEST_AZURE_OPENAI_NOT_SET".into());
    let err = OpenAi::azure(&azure_model(creds, None), &Path::root().key("models").idx(4)).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[4].credentials.api_key");
}
