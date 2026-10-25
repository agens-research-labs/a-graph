use super::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Vertex credentials must be ambient.
#[test]
fn vertex_ambient() {
    // the only option for now: the word is checked by the Loader
    assert!(load(&yml_in(here(), "vertex_ambient")).is_err());
}
