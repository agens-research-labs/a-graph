use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// At `off` nothing is written.
#[tokio::test]
async fn off_shows_nothing() {
    let (mut p, _) = program(here(), "off_shows_nothing", echo).await;
    let screen = show(&mut p, VerboseLevel::Off, true, false);
    p.run("hi").await.unwrap();
    assert_eq!(screen.text(), "");
}
