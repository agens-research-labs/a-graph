use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A call to a tool is an event with its kind, name, duration and result.
#[tokio::test]
async fn tool_call_logged() {
    crate::common::clean_output("target/test-output/tool_call_logged.log");
    let (p, _) = program(here(), "tool_call_logged", model(city(), false)).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("tool_call_logged");
    let call = lines.iter().find(|l| l["code"] == "tool.call").expect("a tool.call event");
    assert_eq!((call["kind"].as_str(), call["name"].as_str(), call["is_error"].as_bool()), (Some("tool"), Some("t"), Some(false)));
    assert!(call["duration"].as_f64().unwrap() >= 0.0);
    assert!(call["result"].as_str().unwrap().contains("Roma"), "{call}");
}
