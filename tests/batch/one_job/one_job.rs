use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The branches of a batch go to the provider as one job.
#[tokio::test]
async fn one_job() {
    let (p, seen, jobs) = program(here(), "one_job", None).await;
    let out = p.run("hi").await.unwrap();

    assert_eq!(job_sizes(&seen), [2], "one job with the requests of d and e");
    assert_eq!(jobs.lock().unwrap().created["b1"].iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>(), ["D(I(S(hi)))", "E(I(S(hi)))"]);

    // the nodes before and after the group are called one by one
    let calls = seen.lock().unwrap().iter().filter(|r| r.method == "POST" && r.path == "/v1/messages").count();
    assert_eq!(calls, 4, "s, i, f and z");

    // the answers went back to the right nodes, in the order of the edges, and `out` got them packed
    let ids: Vec<&str> = out.items.iter().map(|i| i.agent_id.as_str()).collect();
    assert_eq!(ids, ["s", "i", "d", "e", "f", "z"]);
    let text = |i: usize| match &out.items[i].message {
        Some(Content::Text(t)) => t.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(text(2), "D(I(S(hi)))");
    assert_eq!(text(4), "F(---\nfrom agent d\n---\nD(I(S(hi)))\n\n---\nfrom agent e\n---\nE(I(S(hi))))");
}
