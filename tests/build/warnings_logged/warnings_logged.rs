use crate::common::run::*;
use crate::common::{clean_output};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Warnings are sent to the log and do not stop the build.
#[test]
fn warnings_logged() {
    set_env();
    clean_output("target/test-output/skill_warning.log");
    let p = build(&yml(here(), "warnings_logged")).unwrap_or_else(|e| panic!("{e}"));
    // the skill has two fields the standard does not define: one warning each
    assert_eq!(p.warnings.len(), 2);
    let lines = log_lines("skill_warning");
    assert_eq!((lines.len(), lines[0]["level"].as_str(), lines[0]["code"].as_str()), (2, Some("warn"), Some("unknown_field")));
}
