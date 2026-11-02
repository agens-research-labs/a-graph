/// `spec/graph.schema.json` is the schema the code generates. It only checks: after a change of
/// the schema, write it again with `cargo run -- --print-schema > spec/graph.schema.json`.
#[test]
fn up_to_date() {
    let written = std::fs::read_to_string("spec/graph.schema.json").expect("spec/graph.schema.json is missing");
    assert!(
        written == a_graph::schema::json_schema(),
        "spec/graph.schema.json is not the schema of the code: run `cargo run -- --print-schema > spec/graph.schema.json`"
    );
}
