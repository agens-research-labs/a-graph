use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group without out is accepted if its nodes do not pass their output on.
#[test]
fn no_out_ok() {
    assert!(loads(here(), "no_out_ok"));
}
