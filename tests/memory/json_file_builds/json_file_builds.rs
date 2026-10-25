use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A json_file memory with `synt` and `chat` builds and keeps every value.
#[test]
fn json_file_builds() {
    let e = build(here(), "json_file_builds").unwrap();
    let m = e.memory.get("session").unwrap();
    let (s, c) = (m.synt.as_ref().unwrap(), m.chat.as_ref().unwrap());
    assert_eq!((s.length, s.model.as_str(), s.prompt.as_deref(), s.max_tokens), (8, "fast", Some("summary"), 4000));
    assert_eq!((c.length, c.embedding.as_deref()), (6, Some("emb_gemma")));
}
