use crate::common::own_yml;
use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Valid file builds the five managers.
#[test]
fn builds_five() {
    let r = build_text(&own_yml(here())).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(r.prompts.get("analyst").unwrap().body, PromptBody::Content(_)));
    assert!(matches!(&r.prompts.get("chat").unwrap().body, PromptBody::Content(c) if c.system == "You chat."));
    assert!(matches!(
        r.tools.get("get_weather").unwrap().transport,
        ToolTransport::Stdio { .. }
    ));
    assert!(matches!(
        r.mcp.get("remote").unwrap().transport,
        McpTransport::Http { .. }
    ));
    assert_eq!(r.media.get("scan").unwrap().kind, MediaKind::Md);
    assert_eq!(r.media.get("cover").unwrap().kind, MediaKind::Img("png".into()));
    assert!(matches!(r.media.get("cover").unwrap().origin, Origin::Generated { .. }));
    assert_eq!(r.skills.get("pdf").unwrap().name, "pdf");
}
