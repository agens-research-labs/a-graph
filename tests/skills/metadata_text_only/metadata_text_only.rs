use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Metadata must map text to text.
#[test]
fn metadata_text_only() {
    refused(here(), "metadata_text_only", "bad_value", "frontmatter");
}
