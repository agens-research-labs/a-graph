use crate::common::embeddings::*;
use crate::common::run::set_env;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The host of an embedding model that is not in the allow-list is refused.
#[test]
fn host_refused() {
    set_env();
    let e = a_graph::run::build(&own_yml(here())).err().unwrap();
    assert_eq!(e.rules(), ["network_not_allowed"]);
    assert_eq!(e.0[0].path.to_string(), "embeddings[0].host");
}
