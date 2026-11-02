use crate::common::own_yml;
use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The limits of the files have defaults and can be written.
#[test]
fn file_limits_defaults() {
    let r = crate::common::resources::build_text(&own_yml(here())).unwrap();
    let s = r.skills.get("pdf").unwrap();
    assert_eq!((s.max_files, s.max_file_bytes, s.max_depth), (200, 100_000, 6));
    let r = build(here(), "limits").unwrap();
    let s = r.skills.get("pdf").unwrap();
    assert_eq!((s.max_files, s.max_file_bytes, s.max_depth), (2, 10, 1));
    // the limits do what they say: 3 files in 2 levels, cut at 2 and says so; a file over 10 bytes is refused
    let (files, cut) = s.files();
    assert_eq!(
        (files.as_slice(), cut),
        (
            ["assets/blob.bin".to_string(), "references/forms.md".to_string()].as_slice(),
            true
        )
    );
    assert!(
        s.read_file("references/forms.md")
            .unwrap_err()
            .contains("more than the 10 that can be read")
    );
}
