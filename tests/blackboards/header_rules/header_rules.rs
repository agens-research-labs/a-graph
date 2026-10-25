use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The header needs description and purpose with text.
#[test]
fn header_rules() {
    for case in ["no_description", "no_purpose", "empty_description", "empty_purpose", "not_closed", "text_before"] {
        let e = manager(here(), case).unwrap_err();
        assert_eq!(e[0].0, "blackboard_header", "{case}");
    }
}
