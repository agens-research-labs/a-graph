use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The reading of a memory is an event with the id and how many messages came back.
#[tokio::test]
async fn memory_read_logged() {
    crate::common::clean_output("target/test-output/memory_read_logged.log");
    let (p, _) = program(here(), "memory_read_logged", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let lines = crate::common::run::log_lines("memory_read_logged");
    let read = lines
        .iter()
        .find(|l| l["code"] == "read.memory")
        .expect("a read.memory event");
    assert_eq!(
        (
            read["id"].as_str(),
            read["messages"].as_u64(),
            read["summary"].as_bool()
        ),
        (Some("session"), Some(4), Some(false))
    );
}
