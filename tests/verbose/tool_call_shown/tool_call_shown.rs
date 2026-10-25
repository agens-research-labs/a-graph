use crate::common::run_tools::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `detail` shows the tool called, blue, with what it gave back in gray.
#[tokio::test]
async fn tool_call_shown() {
    let (mut p, _) = program(here(), "tool_call_shown", model(city(), false)).await;
    let screen = show(&mut p, VerboseLevel::Detail, true, true);
    p.run("hi").await.unwrap();
    let text = screen.text();
    assert!(text.contains("\x1b[34m⟶ tool t ·"), "{text:?}");
    assert!(text.contains("\x1b[90mresult: "), "{text:?}");
}
