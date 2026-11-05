use super::*;

/// An image given as an address is refused, and the address is not called.
#[tokio::test]
async fn image_url_refused() {
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
    let r = client(&base).invoke(&c).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })), "{r:?}");
    // the host of the address is the server's choice, never checked against `allowed_hosts`
    assert_eq!(seen.lock().unwrap().len(), 1);
}
