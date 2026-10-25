use crate::common::cli::*;

/// `--print-schema`, `--print-docs`, `--print-service-schema`, `--print-service-docs` and
/// `--print-service-api` write the JSON Schemas, the references and the API of the code, and take
/// no file.
#[tokio::test]
async fn print_outputs() {
    let schema = a_graph(&["--print-schema"], None).await;
    assert_eq!(schema.code, 0, "{}", schema.stderr);
    assert!(schema.stdout == a_graph::schema::json_schema());

    let docs = a_graph(&["--print-docs"], None).await;
    assert_eq!(docs.code, 0, "{}", docs.stderr);
    assert!(docs.stdout == a_graph::schema::reference());

    let service_schema = a_graph(&["--print-service-schema"], None).await;
    assert_eq!(service_schema.code, 0, "{}", service_schema.stderr);
    assert!(service_schema.stdout == a_graph::schema::service_json_schema());

    let service_docs = a_graph(&["--print-service-docs"], None).await;
    assert_eq!(service_docs.code, 0, "{}", service_docs.stderr);
    assert!(service_docs.stdout == a_graph::schema::service_reference());

    let api = a_graph(&["--print-service-api"], None).await;
    assert_eq!(api.code, 0, "{}", api.stderr);
    assert!(api.stdout == a_graph::service::openapi());

    let with_file = a_graph(&["--print-schema", "graph.yml"], None).await;
    assert_eq!(with_file.code, 2);
}
