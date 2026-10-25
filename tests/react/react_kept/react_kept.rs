use crate::common::graph_edges::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// React is kept with its controller and its workers.
#[test]
fn react_kept() {
    let (n, e) = build(&yml_in(here(), "react_kept")).unwrap();
    let [Step::React(r)] = e.steps() else { panic!("{:?}", e.steps()) };
    assert_eq!((n.id(r.controller), r.max_iterations, r.workers.len()), (Some("b"), 10, 2));
}
