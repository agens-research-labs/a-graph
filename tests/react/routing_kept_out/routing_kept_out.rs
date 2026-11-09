use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A routing output that is not valid fails the run without what the controller wrote: it is the
/// detail of the error, never its message.
#[tokio::test]
async fn routing_kept_out() {
    let cases = [
        (
            json!({ "done": true, "notes": "the patient has diabetes" }),
            "missing_field",
        ),
        (
            json!({ "done": false, "input": "the patient has diabetes" }),
            "missing_field",
        ),
        (
            json!({ "done": false, "next_worker": "the patient has diabetes", "input": "x" }),
            "unknown_ref",
        ),
    ];
    for (routing, rule) in cases {
        let answer = routing.clone();
        let (p, _) = program(here(), "routing_kept_out", script(move |_, _| answer.clone())).await;
        let e = p.run("hi").await.unwrap_err();
        assert_eq!(e.rule.code(), rule, "{routing}");
        assert!(!e.message.contains("diabetes"), "{}", e.message);
        assert!(e.detail.as_deref().is_some_and(|d| d.contains("diabetes")), "{e:?}");
    }
}
