use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A folder without skill md is an io error.
#[test]
fn no_skill_md() {
    assert_eq!(errors(here(), "no_skill_md")[0].0, "io");
}
