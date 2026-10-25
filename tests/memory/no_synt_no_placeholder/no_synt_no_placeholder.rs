use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory without a summary has no placeholder.
#[test]
fn no_synt_no_placeholder() {
    let e = build(&yml(here(), "no_synt_no_placeholder")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unbound"]);
}
