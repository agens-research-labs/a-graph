use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A web service that takes too long has failed.
#[tokio::test]
async fn http_timeout() {
    // a service that takes the connection and never answers
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tool_port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut held = Vec::new();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            held.push(socket);
        }
    });
    let (p, _) = program_with(here(), "http_timeout", &[("tool", tool_port)], model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: the tool did not answer within 1 s)");
}
