use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A board at a url is refused when the file is checked: it is written as well as read.
#[test]
fn url_board_refused() {
    let errs = manager(here(), "url_board_refused").unwrap_err();
    assert_eq!(
        errs,
        vec![("not_supported".to_string(), "blackboards[1].source.url".to_string())]
    );
}
