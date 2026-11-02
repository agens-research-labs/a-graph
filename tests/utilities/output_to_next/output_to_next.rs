use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The same transformed text reaches the final output and the next node.
#[tokio::test]
async fn output_to_next() {
    let (p, seen) = program(here(), "output_to_next", |req| {
        Resp::ok(chat(&format!("<{}>", user_of(req))))
    })
    .await;
    let out = p.run("hi").await.unwrap();
    let ids = items(&out);
    // a answered `<Q: hi>`; wrapped it is `[<Q: hi>]`, in the output and as the text b got
    assert_eq!(ids[0], pair("a", "[<Q: hi>]"));
    assert_eq!(user_of(&seen.lock().unwrap()[1]), "From a: [<Q: hi>]");
}
