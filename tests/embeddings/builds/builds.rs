use crate::common::embeddings::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A valid file builds the embedding manager, with its host for the network permission.
#[test]
fn builds() {
    let m = manager(&own_yml(here()));
    assert_eq!(m.len(), 1);
    assert!(m.get("emb_gemma").is_some());
    assert_eq!(m.destinations().len(), 1);
}
