pub use super::resources::{build, errors};

/// The message of the first error with this rule, for the skill of the file.
pub fn refused(dir: &std::path::Path, name: &str, rule: &str, about: &str) {
    let r = build(dir, name).unwrap_err();
    assert!(r.0.iter().any(|e| e.rule.code() == rule && e.message.contains(about)), "{r}");
}














