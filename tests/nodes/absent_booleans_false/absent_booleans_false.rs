use crate::common::graph::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Media and message components default absent booleans to false, and `from` of an output to the model.
#[test]
fn absent_booleans_false() {
    let n = build(&own_yml(here())).unwrap();
    let a = n.entity("a").unwrap();
    let media = n.media().get(a).unwrap();
    assert_eq!(media.input.len(), 1);
    assert_eq!((media.input[0].id.as_str(), media.input[0].from, media.input[0].utility.len()), ("scan", a_graph::graph::nodes::component::InFrom::Resources, 0));
    assert!(media.output.is_empty());
    let msg = n.message().get(a).unwrap();
    let i = msg.input.unwrap();
    assert_eq!((i.user_message, i.agent_message), (true, false));
    let o = msg.output.as_ref().unwrap();
    assert_eq!((o.assistant_message, o.node, o.reasoning), (true, true, false));
    // the schema was written as a JSON string and is kept as a JSON object
    assert_eq!(o.schema, Some(serde_json::json!({ "type": "object" })));

    let b = n.entity("b").unwrap();
    let out = &n.media().get(b).unwrap().output[0];
    assert_eq!((out.id.as_str(), out.from, out.assistant, out.node), ("cover", a_graph::graph::nodes::component::OutFrom::Model, false, false));
}
