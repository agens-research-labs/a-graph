use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Vertex host follows the location.
#[test]
fn vertex_host() {
    assert_eq!(
        host(&models(here(), "eu"), "claude_gcp").unwrap(),
        "aiplatform.eu.rep.googleapis.com"
    );
    assert_eq!(
        host(&models(here(), "europe_west1"), "claude_gcp").unwrap(),
        "europe-west1-aiplatform.googleapis.com"
    );
}
