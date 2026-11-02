use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A utility can answer a file: its type and its content in base64.
#[tokio::test]
async fn file_output() {
    let out = call_utility(&utility(), &UtilityInput::Text("x".into())).await.unwrap();
    assert_eq!(
        out,
        UtilityOutput::File {
            mime_type: "application/pdf".into(),
            data: "AAAA".into()
        }
    );
}
