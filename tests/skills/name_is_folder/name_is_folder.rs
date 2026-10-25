use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Name must be the name of the folder.
#[test]
fn name_is_folder() {
    refused(here(), "name_is_folder", "bad_value", "name of the folder");
}
