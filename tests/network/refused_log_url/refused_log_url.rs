use std::time::Duration;

use crate::common::http::{Resp, listen};
use crate::common::yml_with;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A log url whose host is not allowed is never called, so its headers never leave.
#[tokio::test]
async fn refused_log_url() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_REFUSED_LOG_TOKEN", "the-secret") };
    let (port, seen) = listen(|_| Resp::ok("")).await;
    let yml = yml_with(here(), "refused_log_url", &[("sink", port)]);

    let errors = match a_graph::run::build(&yml) {
        Ok(_) => panic!("a log url whose host is not allowed was accepted"),
        Err(e) => e.0,
    };
    assert!(
        errors
            .iter()
            .any(|e| e.rule.code() == "network_not_allowed" && e.path.to_string() == "observability.output.url"),
        "{errors:?}"
    );

    // the log is delivered by a task of its own: it is given the time to send what it had
    tokio::time::sleep(Duration::from_secs(2)).await;
    let seen = seen.lock().unwrap();
    assert!(
        seen.is_empty(),
        "the refused host got {} request(s), the first with the headers:\n{}",
        seen.len(),
        seen[0].headers
    );
}
