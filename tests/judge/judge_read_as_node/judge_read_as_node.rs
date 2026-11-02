use crate::common::graph::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A validation node is read like a node from inside validation node.
#[test]
fn judge_read_as_node() {
    let n = build(&own_yml(here())).unwrap();
    let judge = n.entity("judge").unwrap();
    assert_eq!(n.validation().get(judge).unwrap().on_error, OnError::Stop);
    let m = n.model().get(judge).unwrap();
    assert_eq!((m.id.as_str(), m.max_tokens), ("fast", Some(200)));
    assert_eq!(n.resources().get(judge).unwrap().prompt.as_deref(), Some("check"));
    assert!(n.message().get(judge).unwrap().input.unwrap().user_message);
    // reading is implicit
    let b = n.blackboard().get(judge).unwrap();
    assert_eq!(
        (b.id.as_str(), b.read, b.write.is_none(), b.cleanup),
        ("findings", true, true, false)
    );
    // an ordinary node does not carry `Validation`
    assert!(n.validation().get(n.entity("a").unwrap()).is_none());
}
