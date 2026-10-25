use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Name and description are mandatory.
#[test]
fn name_description_req() {
    refused(here(), "no_name", "missing_field", "name");
    refused(here(), "no_description", "missing_field", "description");
}
