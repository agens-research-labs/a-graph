/// `spec/service.openapi.json` is the description of the API the code generates. It only checks:
/// after a change, write it again with `cargo run -- --print-service-api > spec/service.openapi.json`.
#[test]
fn api_up_to_date() {
    let written = std::fs::read_to_string("spec/service.openapi.json").expect("spec/service.openapi.json is missing");
    assert!(written == a_graph::service::openapi(), "spec/service.openapi.json is not the API of the code: run `cargo run -- --print-service-api > spec/service.openapi.json`");
}
