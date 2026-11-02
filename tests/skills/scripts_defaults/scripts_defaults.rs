use crate::common::own_yml;
use crate::common::resources_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Scripts run by default for 30 seconds unless the skill says otherwise.
#[test]
fn scripts_defaults() {
    let r = crate::common::resources::build_text(&own_yml(here())).unwrap();
    let s = r.skills.get("pdf").unwrap();
    assert_eq!((s.run_scripts, s.timeout, s.max_iter), (true, 30, 20));
    assert_eq!(
        build(here(), "max_iter").unwrap().skills.get("pdf").unwrap().max_iter,
        3
    );
    assert!(crate::common::resources::has(
        here(),
        "max_iter_zero",
        "bad_value",
        "resources.skills[0].max_iter"
    ));
    let r = build(here(), "scripts").unwrap();
    let s = r.skills.get("pdf").unwrap();
    assert_eq!((s.run_scripts, s.timeout), (false, 5));
    assert!(crate::common::resources::has(
        here(),
        "timeout_zero",
        "bad_value",
        "resources.skills[0].timeout"
    ));
}
