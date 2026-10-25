use crate::common::resources::*;

/// A missing block gives empty managers.
#[test]
fn missing_block() {
    let r = Resources::build(None).unwrap();
    assert!(r.prompts.iter().next().is_none() && r.media.iter().next().is_none());
}
