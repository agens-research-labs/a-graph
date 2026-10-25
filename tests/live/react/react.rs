use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The controller of a `react` has to follow the routing output and obey its prompt; the 4 billion
/// parameter model invents the code instead of asking the worker, so this test needs
/// `ollama pull mistral-small3.2` (15 GB), which does it every time.
/// Live a react controller calls its workers and finishes.
#[tokio::test]
#[ignore = "needs a local Ollama with the model mistral-small3.2"]
async fn react() {
    // the controller cannot know the code: it has to ask the worker `vault`
    let out = run_graph(here(), "react", "What is the access code?").await;
    let ids: Vec<&str> = out.items.iter().map(|i| i.agent_id.as_str()).collect();
    assert!(ids.contains(&"vault"), "{ids:?}");
    assert_eq!(&ids[ids.len() - 2..], ["boss", "end"], "{ids:?}");
    let answer = said(out.items.iter().find(|i| i.agent_id == "boss").unwrap());
    assert!(answer.contains("7391"), "{answer}");
}
