use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The last `length` exchanges go to the model as the history, before the new message.
#[tokio::test]
async fn reads_history() {
    let (p, seen) = program(here(), "reads_history", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let roles: Vec<(String, String)> = sent(&seen)["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["role"].as_str().unwrap().into(),
                m["content"].as_str().unwrap().into(),
            )
        })
        .collect();
    let pairs: Vec<(&str, &str)> = roles.iter().map(|(r, c)| (r.as_str(), c.as_str())).collect();
    assert_eq!(
        pairs,
        [
            ("system", "Be brief."),
            ("user", "q2"),
            ("assistant", "a2"),
            ("user", "q3"),
            ("assistant", "a3"),
            ("user", "Question: new")
        ]
    );
}
