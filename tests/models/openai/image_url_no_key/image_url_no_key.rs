use super::*;

/// An image given as an address is downloaded without the key.
#[tokio::test]
async fn image_url_no_key() {
    let own_base = Arc::new(Mutex::new(String::new()));
    let cell = own_base.clone();
    let (base, seen) = serve(move |req| {
        if req.path == "/img/1.png" {
            Resp::ok("PNGDATA")
        } else {
            Resp::ok(json!({ "data": [{ "url": format!("{}/img/1.png", cell.lock().unwrap()) }] }).to_string())
        }
    })
    .await;
    *own_base.lock().unwrap() = base.clone();

    let mut c = call("x");
    c.output_media = vec![image("png", generate(), None)];
    let r = client(&base).invoke(&c).await.unwrap();
    assert_eq!(
        (r.media[0].mime_type.as_str(), r.media[0].data.as_str()),
        ("image/png", STANDARD.encode("PNGDATA").as_str())
    );

    let requests = seen.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].headers.contains("authorization: bearer test-key"));
    // the address is anyone's: the API key must not go there
    assert_eq!(requests[1].path, "/img/1.png");
    assert!(
        !requests[1].headers.contains("authorization") && !requests[1].headers.contains("test-key"),
        "{}",
        requests[1].headers
    );
}
