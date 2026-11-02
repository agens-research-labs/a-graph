use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media has one writer and must have generate.
#[test]
fn media_one_writer() {
    assert!(has(
        here(),
        "media_two_writers",
        "media_two_writers",
        "graph.nodes[2].media.output[0]"
    ));
    assert!(has(
        here(),
        "media_not_generated",
        "media_not_generated",
        "graph.nodes[1].media.output[0]"
    ));
}
