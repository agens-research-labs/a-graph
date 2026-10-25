use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The provider family of a media must be the family of the model.
#[test]
fn media_family() {
    assert!(has(here(), "media_family", "not_supported", "graph.nodes[1].model"));
    assert_eq!(errors(here(), "media_family").len(), 1, "one error, not two");
}
