use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Both transports are read with every value; a header is kept as the name of its variable.
#[test]
fn builds() {
    // SAFETY: the name of the variable the yml writes; it is never used to call anything.
    unsafe { std::env::set_var("A_GRAPH_TEST_UTIL_TOKEN", "t") };
    let m = manager(&yml_with(here(), "builds", &[("utility", 19901)]));
    assert_eq!(m.len(), 2);
    let a = m.get("to_md").unwrap();
    assert!(
        matches!(&a.transport, UtilityTransport::Stdio { command, args } if command == "python3" && args == &["u.py", "-x"])
    );
    assert_eq!(a.timeout, 60);
    let b = m.get("ocr").unwrap();
    assert!(
        matches!(&b.transport, UtilityTransport::Http { url, headers } if url == "http://127.0.0.1:19901/ocr" && headers["Authorization"] == "A_GRAPH_TEST_UTIL_TOKEN")
    );
    assert_eq!(m.destinations().len(), 1, "only the http one reaches the network");
}
