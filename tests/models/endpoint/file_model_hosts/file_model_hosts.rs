use super::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Hosts of the file models.
#[test]
fn file_model_hosts() {
    let m = models_of(&own_yml(here()));
    let hosts: Vec<String> = ["fast", "strong", "smart", "claude_gcp", "claude_azure", "local"].iter().map(|id| host(&m, id).unwrap()).collect();
    assert_eq!(
        hosts,
        [
            "api.anthropic.com",
            "bedrock-runtime.eu-west-1.amazonaws.com",
            "api.openai.com",
            "aiplatform.googleapis.com",
            "my-res.services.ai.azure.com",
            "localhost"
        ]
    );
}
