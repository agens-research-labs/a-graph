use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// A run that does not exist answers 404, for its state and for its events.
#[tokio::test]
async fn runs_unknown() {
    let s = start(&config_path(), 19969).await;
    assert_eq!(s.get("/runs/r-nothing").await.0, 404);
    assert_eq!(s.get("/runs/r-nothing/events").await.0, 404);
}
