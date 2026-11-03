use crate::common::blackboards::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Cleanup empties a memory board.
#[tokio::test]
async fn cleanup_memory() {
    let m = build_text(&own_yml(here())).unwrap();
    m.write("findings", "text", Place::Bottom).await.unwrap();
    m.cleanup("findings").await.unwrap();
    assert_eq!(m.read("findings").unwrap(), "");
}
