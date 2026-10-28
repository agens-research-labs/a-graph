use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The host checked is the host the HTTP client calls: a URL is read as the `url` crate reads it.
#[test]
fn url_parsed() {
    let trace = ("network_not_allowed".to_string(), "trace.endpoint".to_string());
    assert_eq!(errors(here(), "backslash"), vec![trace.clone()]);
    assert_eq!(errors(here(), "userinfo"), vec![trace]);
    let bad = |path: &str| ("bad_value".to_string(), path.to_string());
    assert_eq!(errors(here(), "blanks"), vec![bad("observability.output.url"), bad("trace.endpoint")]);
}
