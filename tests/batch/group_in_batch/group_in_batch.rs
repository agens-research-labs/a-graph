use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group inside a batch is refused when loading: the branches of a batch are plain nodes.
#[test]
fn group_in_batch() {
    set_env();
    // SAFETY: the name of the variable the yml writes; it is never used to call anything.
    unsafe { std::env::set_var("A_GRAPH_TEST_RUN_BATCH_KEY", "k") };
    let e = build(&yml(here(), "group_in_batch")).err().unwrap();
    assert_eq!(e.rules(), ["batch_nested"]);
}
