use crate::common::graph::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Every node is an entity with the components it declares.
#[test]
fn entity_components() {
    let n = build(&own_yml(here())).unwrap();
    assert_eq!(n.len(), 8);
    let [a, b, c, judge] = ["a", "b", "c", "judge"].map(|id| n.entity(id).unwrap());
    assert_eq!(n.id(a), Some("a"));
    assert_eq!(n.entities().count(), 8);
    assert!(n.entity("ghost").is_none());

    // `a` has everything of a regular node and no validation
    assert!(n.model().get(a).is_some() && n.external().get(a).is_some() && n.resources().get(a).is_some());
    assert!(n.blackboard().get(a).is_some() && n.media().get(a).is_some() && n.message().get(a).is_some());
    assert!(n.validation().get(a).is_none());
    // `c` only has a model; the judge has the components it reads with, and `Validation`
    assert!(n.model().get(c).is_some() && n.message().get(c).is_none() && n.resources().get(c).is_none());
    assert!(
        n.model().get(judge).is_some() && n.resources().get(judge).is_some() && n.validation().get(judge).is_some()
    );
    assert!(n.media().get(judge).is_none());
    // the components are listed by entity
    assert_eq!(n.model().iter().count(), 8, "every node has a model");
    assert_eq!(n.validation().iter().count(), 1);
    let _ = b;
}
