use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A Bedrock batch also reaches the control API and the S3 buckets: their hosts must be allowed.
#[test]
fn batch_hosts() {
    assert_eq!(errors(here(), "allowed"), Vec::<(String, String)>::new());
    let refused: Vec<String> = errors(here(), "refused")
        .into_iter()
        .filter(|(r, _)| r == "network_not_allowed")
        .map(|(_, p)| p)
        .collect();
    assert_eq!(
        refused,
        [
            "models[1].batch",
            "models[1].batch.input_s3_uri",
            "models[1].batch.output_s3_uri"
        ]
    );
    // an input that is not an s3:// URI has no host to check
    assert_eq!(
        errors(here(), "bad_uri"),
        [("bad_value".to_string(), "models[1].batch.input_s3_uri".to_string())]
    );
}
