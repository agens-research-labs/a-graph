use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A url without a host is a bad value.
#[test]
fn url_no_host() {
    assert!(errors(here(), "no_host").contains(&("bad_value".into(), "observability.output.url".into())));
    // a local host that is not a URL
    assert!(errors(here(), "local_not_url").contains(&("bad_value".into(), "models[5]".into())));
}
