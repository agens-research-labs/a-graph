/// `docs/service.md` is the reference of the file of the service the code generates. It only
/// checks: after a change, write it again with `cargo run -- --print-service-docs > docs/service.md`.
#[test]
fn service_docs_up_to_date() {
    let written = std::fs::read_to_string("docs/service.md").expect("docs/service.md is missing");
    assert!(
        written == a_graph::schema::service_reference(),
        "docs/service.md is not the reference of the code: run `cargo run -- --print-service-docs > docs/service.md`"
    );
}
