use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Name follows the standard.
#[test]
fn name_rules() {
    refused(here(), "name_uppercase", "bad_value", "lowercase");
    refused(here(), "name_leading_hyphen", "bad_value", "hyphen");
    refused(here(), "name_double_hyphen", "bad_value", "hyphen");
    refused(here(), "name_too_long", "bad_value", "64");
}
