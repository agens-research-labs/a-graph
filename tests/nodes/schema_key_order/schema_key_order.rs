use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The schema reaches the model with its keys in the order they are written, as yml or as a
/// JSON string; the JSON answer keeps the order the model wrote.
#[tokio::test]
async fn schema_key_order() {
    for name in ["schema_key_order", "json_string"] {
        let answer = chat("{\"solution\": \"s\", \"answer\": \"1\"}", None);
        let (p, seen) = program(here(), name, Resp::ok(answer)).await;
        let out = p.run_node(entity(&p), user("q")).await.unwrap();
        let body = seen.lock().unwrap()[0].body.clone();
        let format = &body[body.find("\"format\"").unwrap()..];
        assert!(
            format.find("\"solution\"").unwrap() < format.find("\"answer\"").unwrap(),
            "{name}: {body}"
        );
        let Some(Content::Json(j)) = out.assistant.message else {
            panic!("{name}: no JSON answer")
        };
        assert_eq!(j.to_string(), r#"{"solution":"s","answer":"1"}"#, "{name}");
    }
}
