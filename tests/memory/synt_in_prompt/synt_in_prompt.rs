use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The summary of the record goes where the prompt writes the id of the memory.
#[tokio::test]
async fn synt_in_prompt() {
    let (p, seen) = program(here(), "synt_in_prompt", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let messages = sent(&seen)["messages"].clone();
    let last = messages.as_array().unwrap().last().unwrap();
    assert_eq!(last["content"], "Summary: The user asks about ETFs. / Question: new");
}
