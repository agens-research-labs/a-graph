use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A depth of zero looks only at the files of the folder itself.
#[test]
fn depth_zero() {
    let r = build(here(), "depth_zero").unwrap();
    let mut s = r.skills.get("pdf").unwrap().clone();
    (s.max_files, s.max_depth) = (200, 0);
    assert_eq!(
        s.files(),
        (Vec::<String>::new(), false),
        "everything in the folder is in sub-folders"
    );
}
