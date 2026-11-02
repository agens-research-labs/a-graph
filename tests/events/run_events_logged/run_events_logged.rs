use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The run writes its events in the log, in order, with the fields of each.
#[tokio::test]
async fn run_events_logged() {
    crate::common::clean_output("target/test-output/run_events_logged.log");
    let (p, _) = program(here(), "run_events_logged", echo).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("run_events_logged");
    let codes: Vec<&str> = lines.iter().map(|l| l["code"].as_str().unwrap()).collect();
    assert_eq!(codes.first(), Some(&"run.start"));
    assert_eq!(codes.last(), Some(&"run.end"));
    for wanted in ["node.start", "node.end", "model.call", "group.start", "group.end"] {
        assert!(codes.contains(&wanted), "`{wanted}` not in {codes:?}");
    }
    let end = lines
        .iter()
        .find(|l| l["code"] == "node.end" && l["node"] == "s")
        .unwrap();
    assert_eq!(
        (
            end["model"].as_str(),
            end["input_tokens"].as_u64(),
            end["output_tokens"].as_u64()
        ),
        (Some("m"), Some(1), Some(1))
    );
    assert!(end["duration"].as_f64().unwrap() >= 0.0);
    let group = lines.iter().find(|l| l["code"] == "group.start").unwrap();
    assert_eq!(
        (group["pattern"].as_str(), group["branches"].as_u64()),
        (Some("parallel"), Some(2))
    );
    // the contents of the messages are in the log because the data is public
    let call = lines.iter().find(|l| l["code"] == "model.call").unwrap();
    assert_eq!(call["prompt"], "S(hi)");
}
