use crate::common::graph::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The model component keeps the parameters of the node.
#[test]
fn model_component() {
    let n = build(&own_yml(here())).unwrap();
    let m = n.model().get(n.entity("a").unwrap()).unwrap();
    assert_eq!(
        (m.id.as_str(), m.max_tokens, m.reasoning),
        ("fast", Some(1000), Some(Reasoning::Low))
    );
    assert_eq!((m.temperature, m.top_p), (None, None));
}
