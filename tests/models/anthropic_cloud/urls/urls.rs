use super::*;

/// The urls follow the location and the resource.
#[test]
fn urls() {
    let at = Path::root();
    let url = |loc: &str| Anthropic::vertex(&vertex_model(loc), &at).unwrap().endpoint();
    assert_eq!(url("global"), "https://aiplatform.googleapis.com/v1/projects/my-project/locations/global/publishers/anthropic/models/claude-sonnet-5-5:rawPredict");
    assert_eq!(url("eu"), "https://aiplatform.eu.rep.googleapis.com/v1/projects/my-project/locations/eu/publishers/anthropic/models/claude-sonnet-5-5:rawPredict");
    assert_eq!(url("us-east5"), "https://us-east5-aiplatform.googleapis.com/v1/projects/my-project/locations/us-east5/publishers/anthropic/models/claude-sonnet-5-5:rawPredict");

    let azure = Anthropic::azure(&azure_model(AzureCredentials::Ambient), &at).unwrap();
    assert_eq!(azure.endpoint(), "https://my-res.services.ai.azure.com/anthropic/v1/messages");
}
