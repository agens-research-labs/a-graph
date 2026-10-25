use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The logging destinations must be allowed.
#[test]
fn logging_hosts() {
    assert!(refused(here(), "url", "observability.output.url"));
    assert!(refused(here(), "trace", "trace.endpoint"));
}
