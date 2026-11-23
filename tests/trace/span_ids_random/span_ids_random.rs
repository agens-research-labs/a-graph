use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Two programs built from the same graph, as the service builds one for each request, share no span id: the ids
/// are random, so two runs that end up in the same trace can be told apart.
#[tokio::test]
async fn span_ids_random() {
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let (first, _) = program_with(here(), "span_ids_random", &[("collector", port)], echo).await;
    let (second, _) = program_with(here(), "span_ids_random", &[("collector", port)], echo).await;
    first.run("hi").await.unwrap();
    let seen = spans_of(&collector).len();
    second.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let id = |s: &Value| s["spanId"].as_str().unwrap().to_string();
    let ids_first: std::collections::HashSet<String> = spans[..seen].iter().map(id).collect();
    let ids_second: std::collections::HashSet<String> = spans[seen..].iter().map(id).collect();
    assert!(seen > 1 && spans.len() > seen, "both runs sent their spans: {spans:?}");
    assert_eq!(ids_first.len(), seen, "the ids of a run are all different");
    assert!(
        ids_first.is_disjoint(&ids_second),
        "the two runs share span ids: {ids_first:?} {ids_second:?}"
    );
    for s in &spans {
        let id = id(s);
        assert_eq!(id.len(), 16, "an id is 16 hex digits: {id}");
        assert_ne!(id, "0000000000000000", "an id is never 0");
    }
}
