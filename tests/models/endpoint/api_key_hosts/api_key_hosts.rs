use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Hosts of the other api key providers.
#[test]
fn api_key_hosts() {
    let m = models(here(), "api_key_hosts");
    assert_eq!(host(&m, "g").unwrap(), "generativelanguage.googleapis.com");
    assert_eq!(host(&m, "mi").unwrap(), "api.mistral.ai");
}
