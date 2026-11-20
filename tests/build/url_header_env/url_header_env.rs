use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A header of the url of a media whose variable is not set is reported when the file is checked, before anything
/// runs.
#[test]
fn url_header_env() {
    assert_eq!(
        errors(here(), "url_header_env"),
        [(
            "env_missing".to_string(),
            "resources.media[0].source.headers.Authorization".to_string()
        )]
    );
}
