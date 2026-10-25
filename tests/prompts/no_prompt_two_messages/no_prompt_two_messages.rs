use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that takes both messages needs a prompt: without it the model could not tell them apart.
#[test]
fn no_prompt_two_messages() {
    let e = build(&yml(here(), "no_prompt_two_messages")).err().unwrap();
    assert_eq!(e.rules(), ["prompt_required"]);
}
