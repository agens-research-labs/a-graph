use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Search keeps what was written.
#[test]
fn search_keeps_values() {
    let e = build(here(), "search_keeps_values").unwrap();
    let s = e.search.get("docs_index").unwrap();
    assert_eq!(
        (s.provider.as_str(), s.url.as_str()),
        ("elasticsearch", "https://s.example.com")
    );
    assert!(s.access.is_some());
}
