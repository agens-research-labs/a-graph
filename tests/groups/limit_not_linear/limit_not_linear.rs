use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Limit is not for linear.
#[test]
fn limit_not_linear() {
    assert!(has(
        here(),
        "limit_not_linear",
        "unexpected_field",
        "graph.edges[2].parallel[2].fan[1].linear.limit"
    ));
}
