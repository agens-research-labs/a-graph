use crate::common::graph::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// External resources and blackboard components.
#[test]
fn external_board_parts() {
    let n = build(&own_yml(here())).unwrap();
    let a = n.entity("a").unwrap();
    let ext = n.external().get(a).unwrap();
    assert_eq!((ext.vectordb[0].id.as_str(), &ext.vectordb[0].query), ("kb", &Query::UserMessage));
    let r = n.resources().get(a).unwrap();
    assert_eq!(r.prompt.as_deref(), Some("analyst"));
    assert_eq!(r.tools, ["get_weather"]);
    assert_eq!(r.skills, ["pdf"]);
    let b = n.blackboard().get(a).unwrap();
    assert!(b.read && !b.cleanup);
    assert_eq!(b.write.as_ref().map(|w| (w.enabled, w.place)), Some((true, Place::Top)));
}
