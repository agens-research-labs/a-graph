use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node of a batch job with a `schema_prompt` is refused: a batch cannot send a correction. The `in` and `out`
/// nodes are not in the job, and may have one.
#[test]
fn schema_prompt_refused() {
    assert_eq!(
        errors(here(), "schema_prompt_refused"),
        [(
            "batch_schema_prompt".to_string(),
            "graph.nodes[2].message.output.schema_prompt".to_string()
        )]
    );
}
