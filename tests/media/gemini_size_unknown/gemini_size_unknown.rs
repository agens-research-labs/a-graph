use crate::common::resources::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An unknown gemini image size is refused by the loader.
#[test]
fn gemini_size_unknown() {
    assert!(load(&yml_in(here(), "gemini_size_unknown")).is_err());
}
