use crate::common::run_live::*;
use crate::common::utilities::{UtilityInput, UtilityOutput, call_utility, manager};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live: a Python utility (pymupdf4llm) turns a real PDF into Markdown, and a node on Ollama answers from it.
#[tokio::test]
#[ignore = "needs python3 with pymupdf4llm and a local Ollama with the model qwen3.5:4b"]
async fn live_pdf_to_md() {
    // the utility alone: the PDF as Markdown, with its heading
    let u = manager(&crate::common::yml_in(here(), "live_pdf_to_md"))
        .get("pdf_to_md")
        .unwrap()
        .clone();
    let path = "tests/utilities/live_pdf_to_md/files/contract.pdf".to_string();
    let UtilityOutput::Text(md) = call_utility(
        &u,
        &UtilityInput::File {
            path: path.clone(),
            mime_type: Some("application/pdf".into()),
        },
    )
    .await
    .unwrap() else {
        panic!("a text was expected")
    };
    assert!(
        md.starts_with("# Lease agreement") && md.contains("12 months") && md.contains("800 euro"),
        "{md}"
    );

    // the whole path: PDF -> utility -> Markdown -> model
    let out = ask(
        &program(here(), "live_pdf_to_md"),
        "How many months does the lease last? Answer with just the number.",
    )
    .await;
    assert!(text(&out.assistant.message).contains("12"));
}
