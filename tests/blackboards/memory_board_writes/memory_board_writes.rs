use crate::common::blackboards::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory board starts empty and takes writes.
#[tokio::test]
async fn memory_board_writes() {
    let m = build_text(&own_yml(here())).unwrap();
    assert_eq!(m.read("findings").unwrap(), "");
    m.write("findings", "first", Place::Bottom).await.unwrap();
    m.write("findings", "second", Place::Bottom).await.unwrap();
    m.write("findings", "zero", Place::Top).await.unwrap();
    assert_eq!(m.read("findings").unwrap(), "zero\n\nfirst\n\nsecond\n");
}
