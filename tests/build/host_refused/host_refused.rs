use crate::common::clean_output;
use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A host that is not allowed is returned and logged.
#[test]
fn host_refused() {
    set_env();
    clean_output("target/test-output/host_not_allowed.log");
    let e = build(&yml(here(), "host_refused")).err().unwrap();
    assert_eq!(e.rules(), ["network_not_allowed"]);
    let lines = log_lines("host_not_allowed");
    assert_eq!(
        (lines.len(), lines[0]["code"].as_str(), lines[0]["path"].as_str()),
        (1, Some("network_not_allowed"), Some("models[5]"))
    );
}
