/// `docs/graph.md` is the reference the code generates. It only checks: after a change of the
/// schema, write it again with `cargo run -- --print-docs > docs/graph.md`.
#[test]
fn docs_up_to_date() {
    let written = std::fs::read_to_string("docs/graph.md").expect("docs/graph.md is missing");
    assert!(
        written == a_graph::schema::reference(),
        "docs/graph.md is not the reference of the code: run `cargo run -- --print-docs > docs/graph.md`"
    );
}
