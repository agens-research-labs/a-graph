use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Http tools, prompt urls, media urls and search are destinations.
#[test]
fn destinations_listed() {
    assert!(refused(here(), "http_tool", "resources.tools[0].url"));
    assert!(refused(here(), "prompt_url", "resources.prompts[1].source.url"));
    assert!(refused(here(), "media_url", "resources.media[0].source.url"));
    assert!(refused(here(), "search_url", "external.search[0].url"));
}
