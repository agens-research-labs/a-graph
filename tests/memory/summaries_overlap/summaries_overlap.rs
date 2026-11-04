use crate::common::fresh_copy;
use crate::common::http::listen_held;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The last message sent to the mock.
fn last_text(r: &Req) -> String {
    let body: Value = serde_json::from_str(&r.body).unwrap();
    body["messages"].as_array().unwrap().last().unwrap()["content"]
        .as_str()
        .unwrap()
        .to_string()
}

fn is_summary(r: &Req) -> bool {
    last_text(r).starts_with("Previous summary")
}

/// Run A (`q2`) asks for a summary, held by the mock; run B (`q3`) runs to its end meanwhile and writes its
/// summary, which holds both exchanges. Then the summary of A comes: it is dropped, and no exchange is lost.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn summaries_overlap() {
    let path = "target/test-memory/summaries_overlap.json";
    fresh_copy(here(), "files/session.json", path);
    let (model_port, seen, release) = listen_held(
        |r| is_summary(r) && !last_text(r).contains("user: q3"),
        |r| {
            let text = last_text(r);
            let answer = match (is_summary(r), text.contains("user: q3")) {
                (true, true) => "SUMMARY B",
                (true, false) => "SUMMARY A",
                (false, _) if text.ends_with("q2") => "a2",
                (false, _) => "a3",
            };
            Resp::ok(chat(answer, None))
        },
    )
    .await;
    let yml = yml_with(here(), "summaries_overlap", &[("model", model_port)]);

    let a = build(&yml).unwrap_or_else(|e| panic!("{e}"));
    let run_a = tokio::spawn(async move { a.run_node(entity(&a), user("q2")).await });
    let at = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !seen.lock().unwrap().iter().any(is_summary) {
        assert!(std::time::Instant::now() < at, "run A never asked for its summary");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let b = build(&yml).unwrap_or_else(|e| panic!("{e}"));
    b.run_node(entity(&b), user("q3")).await.unwrap();
    release.notify_one();
    run_a.await.unwrap().unwrap();

    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        (
            records[0]["synt"].as_str(),
            records[0]["chat"].as_array().unwrap().len()
        ),
        (Some("SUMMARY B"), 0),
        "the summary of B holds every exchange; the one of A, made before `q3`, is dropped"
    );
}
