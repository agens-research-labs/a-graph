use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Nodes under a batch with one model that supports it build.
#[test]
fn batch_builds() {
    set_env();
    build(&yml(here(), "batch_builds")).unwrap_or_else(|e| panic!("{e}"));
}
