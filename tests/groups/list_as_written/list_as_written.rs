use crate::common::graph_edges::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The list is kept as written with the groups nested.
#[test]
fn list_as_written() {
    let (n, e) = build(&own_yml(here())).unwrap();
    assert_eq!((n.id(e.start()), e.end().and_then(|x| n.id(x))), (Some("a"), Some("g")));
    let [Step::Node(judge), Step::Group(parallel)] = e.steps() else {
        panic!("{:?}", e.steps())
    };
    assert_eq!(n.id(*judge), Some("judge"));
    assert_eq!(
        (
            parallel.pattern,
            parallel.limit,
            n.id(parallel.r#in),
            parallel.out.and_then(|o| n.id(o))
        ),
        (Pattern::Parallel, Some(20), Some("b"), Some("f"))
    );
    let [Step::Node(c), Step::Group(linear)] = parallel.fan.as_slice() else {
        panic!("{:?}", parallel.fan)
    };
    assert_eq!(n.id(*c), Some("c"));
    assert_eq!(
        (linear.pattern, n.id(linear.r#in), linear.out),
        (Pattern::Linear, Some("d"), None)
    );
    assert!(matches!(linear.fan.as_slice(), [Step::Node(x)] if n.id(*x) == Some("e")));
}
