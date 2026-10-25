use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Fields the standard does not define are ignored with a warning.
#[test]
fn extra_fields_warn() {
    let r = build(here(), "extra_fields_warn").unwrap();
    let w = r.skills.warnings();
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].code, "unknown_field");
    assert!(w[0].message.contains("argument-hint"));
    assert!(r.skills.get("pdf").is_some());
}
