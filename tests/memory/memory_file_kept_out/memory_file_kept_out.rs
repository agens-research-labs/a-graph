use crate::common::fresh_copy;
use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

const FILE: &str = "target/test-memory/memory_file_kept_out.json";

/// A memory file that is not a list of records fails the node with where, never with what the file
/// holds: that is the detail, and it stays out of the log when the passport forbids contents. The
/// file is read before the model is called, and again when the exchange is saved: in the second
/// case it goes bad while the model answers.
#[tokio::test]
async fn memory_file_kept_out() {
    for bad_while_answering in [false, true] {
        crate::common::clean_output("target/test-output/memory_file_kept_out.log");
        if bad_while_answering {
            crate::common::clean_output(FILE);
        } else {
            fresh_copy(here(), "files/session.json", FILE);
        }
        let (port, _) = listen(move |req| {
            if bad_while_answering {
                fresh_copy(here(), "files/session.json", FILE);
            }
            echo(req)
        })
        .await;
        let yml = yml_with(here(), "memory_file_kept_out", &[("model", port)]);
        let p = Shared::new(build(&yml).unwrap_or_else(|e| panic!("{e}")));
        let e = p.run("hi").await.unwrap_err();
        let case = if bad_while_answering { "saving" } else { "reading" };
        assert_eq!(e.rule.code(), "io", "{case}: {e:?}");
        assert!(
            e.message.contains("is not a list of records (line 2, column") && !e.message.contains("diabetes"),
            "{case}: {}",
            e.message
        );
        assert!(
            e.detail.as_deref().is_some_and(|d| d.contains("diabetes")),
            "{case}: {e:?}"
        );

        let log = std::fs::read_to_string("target/test-output/memory_file_kept_out.log").unwrap();
        assert!(!log.contains("diabetes"), "{case}: {log}");
        let lines = crate::common::run::log_lines("memory_file_kept_out");
        let error = lines.iter().find(|l| l["code"] == "io").unwrap();
        assert_eq!(error["content_redacted"], true, "{case}: {error}");
    }
}
