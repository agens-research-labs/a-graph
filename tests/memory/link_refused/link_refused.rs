use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory that is a symbolic link, here to a file that does not exist: the write is refused, the link
/// stays and no file is created where it points.
#[cfg(unix)]
#[tokio::test]
async fn link_refused() {
    let (link, target) = (
        "target/test-memory/link_refused.json",
        "target/test-memory/link_refused.target.json",
    );
    clean_output(link);
    clean_output(target);
    std::os::unix::fs::symlink("link_refused.target.json", link).unwrap();
    let (p, _) = program(here(), "link_refused", Resp::ok(chat("the answer", None))).await;
    let e = p.run_node(entity(&p), user("the question")).await.unwrap_err();
    assert!(e.to_string().contains("symbolic link"), "{e}");
    assert!(std::fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert!(!std::path::Path::new(target).exists());
}
