/// `spec/service.schema.json` is the schema of the file of the service the code generates. It only
/// checks: after a change, write it again with `cargo run -- --print-service-schema > spec/service.schema.json`.
#[test]
fn service_up_to_date() {
    let written = std::fs::read_to_string("spec/service.schema.json").expect("spec/service.schema.json is missing");
    assert!(
        written == a_graph::schema::service_json_schema(),
        "spec/service.schema.json is not the schema of the code: run `cargo run -- --print-service-schema > spec/service.schema.json`"
    );
}
