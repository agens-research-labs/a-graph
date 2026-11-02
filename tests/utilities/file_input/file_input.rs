use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A file goes in as its path and, when known, its type.
#[tokio::test]
async fn file_input() {
    let u = utility();
    let known = call_utility(
        &u,
        &UtilityInput::File {
            path: "docs/a.pdf".into(),
            mime_type: mime_of("docs/a.pdf"),
        },
    )
    .await
    .unwrap();
    assert_eq!(known, UtilityOutput::Text("docs/a.pdf|application/pdf".into()));
    let unknown = call_utility(
        &u,
        &UtilityInput::File {
            path: "docs/a.xyz".into(),
            mime_type: mime_of("docs/a.xyz"),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        unknown,
        UtilityOutput::Text("docs/a.xyz|".into()),
        "no type is sent when the extension does not say it"
    );
}
