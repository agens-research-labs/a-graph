use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An id that is not in `utilities` is an unknown reference.
#[test]
fn unknown_in_message() {
    let e = a_graph::loader::load(&yml_in(here(), "unknown_in_message")).unwrap_err();
    assert!(
        e.0.iter()
            .any(|x| x.rule.code() == "unknown_ref" && x.path.to_string() == "graph.nodes[0].message.input.utility[1]"),
        "{e:?}"
    );
}
