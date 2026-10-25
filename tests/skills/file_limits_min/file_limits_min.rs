fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The limits of the files are at least one.
#[test]
fn file_limits_min() {
    for what in ["max_files", "max_file_bytes", "max_depth"] {
        assert!(crate::common::resources::has(here(), "file_limits_min", "bad_value", &format!("resources.skills[0].{what}")), "{what}");
    }
}
