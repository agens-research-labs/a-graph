use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file without frontmatter is refused.
#[test]
fn no_frontmatter() {
    refused(here(), "no_frontmatter", "bad_value", "frontmatter");
}
