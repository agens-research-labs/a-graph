use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Description and compatibility have a length limit.
#[test]
fn text_length_limits() {
    refused(here(), "description", "bad_value", "1024");
    refused(here(), "compatibility", "bad_value", "500");
}
