use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A local host must be a url with a host.
#[test]
fn local_host_url() {
    assert!(host(&models(here(), "local_host_url"), "local").is_err());
}
