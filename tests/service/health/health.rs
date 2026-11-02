use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// `GET /health` answers 200.
#[tokio::test]
async fn health() {
    let s = start(&config_path()).await;
    let r = reqwest::get(format!("http://127.0.0.1:{}/health", s.port))
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    assert_eq!(r.json::<Value>().await.unwrap(), json!({ "status": "ok" }));
}
