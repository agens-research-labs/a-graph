use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media with `generate` is produced by a node: listing it in `media.input.ids` is refused when loading.
#[test]
fn media_input_generated() {
    assert!(has(
        here(),
        "media_input_generated",
        "media_input_generated",
        "graph.nodes[0].media.input[1]"
    ));
}
