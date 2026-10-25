use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A valid skill keeps its metadata.
#[test]
fn keeps_metadata() {
    let r = build(here(), "keeps_metadata").unwrap();
    let s = r.skills.get("pdf").unwrap();
    assert_eq!((s.name.as_str(), s.description.as_str()), ("pdf", "Does PDFs."));
    assert_eq!(s.license.as_deref(), Some("MIT"));
    assert_eq!(s.metadata["version"], "1.0");
    assert_eq!(s.allowed_tools.as_deref(), Some("Read"));
    assert!(r.skills.warnings().is_empty(), "a standard skill has no warnings");
}
