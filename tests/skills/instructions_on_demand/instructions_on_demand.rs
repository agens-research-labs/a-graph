use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The instructions are read on demand.
#[test]
fn instructions_on_demand() {
    let r = crate::common::resources::build_text(&own_yml(here())).unwrap();
    assert_eq!(r.skills.get("pdf").unwrap().instructions().unwrap(), "\n# PDF\n\nStep 1: read the file.\n");
}
