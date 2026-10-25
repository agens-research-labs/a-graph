use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The host of an http utility must be in the allow-list of the passport.
#[test]
fn host_refused() {
    use crate::common::run::set_env;
    set_env();
    let e = a_graph::run::build(&own_yml(here())).err().unwrap();
    assert_eq!(e.rules(), ["network_not_allowed"]);
    assert_eq!(e.0[0].path.to_string(), "utilities[0].url");
}
