use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Azure host follows the publisher.
#[test]
fn azure_host() {
    assert_eq!(
        host(&models(here(), "azure_host"), "claude_azure").unwrap(),
        "my-res.openai.azure.com"
    );
}
