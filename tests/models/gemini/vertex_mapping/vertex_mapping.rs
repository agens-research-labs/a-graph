use super::*;

/// Vertex uses the same mapping with a bearer token and its own url.
#[tokio::test]
async fn vertex_mapping() {
    let at = Path::root();
    assert_eq!(
        Gemini::vertex(&vertex_model("global"), &at).unwrap().endpoint(),
        "https://aiplatform.googleapis.com/v1beta1/projects/my-project/locations/global/interactions"
    );
    assert_eq!(
        Gemini::vertex(&vertex_model("eu"), &at).unwrap().endpoint(),
        "https://aiplatform.eu.rep.googleapis.com/v1beta1/projects/my-project/locations/eu/interactions"
    );
    assert_eq!(
        Gemini::vertex(&vertex_model("europe-west1"), &at).unwrap().endpoint(),
        "https://europe-west1-aiplatform.googleapis.com/v1beta1/projects/my-project/locations/europe-west1/interactions"
    );

    let (base, seen) = serve(|_| Resp::ok(text_answer("hello"))).await;
    let m = Gemini::vertex(&vertex_model("global"), &at)
        .unwrap()
        .with_interactions_url(format!("{base}/v1beta1/interactions"))
        .with_token_source(Box::new(StaticToken("goog-token".into())));
    let r = m.invoke(&call("q")).await.unwrap();
    assert_eq!(
        (r.message.as_deref(), r.info.provider.as_str()),
        (Some("hello"), "vertex_ai")
    );
    let req = seen.lock().unwrap()[0].clone();
    assert!(
        req.headers.contains("authorization: bearer goog-token") && !req.headers.contains("x-goog-api-key"),
        "{}",
        req.headers
    );
    assert_eq!(body(&req)["model"], "gemini-3.5-flash-lite");
}
