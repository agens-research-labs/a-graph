use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A header that names an environment variable that is not set is an error when the program is built.
#[test]
fn header_env_missing() {
    let e = a_graph::run::build(&yml_in(here(), "header_env_missing"))
        .err()
        .unwrap();
    assert_eq!(e.rules(), ["env_missing"]);
    assert_eq!(e.0[0].path.to_string(), "trace.headers.Authorization");
}
