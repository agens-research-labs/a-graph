use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Secret must be an env reference.
#[test]
fn secret_env_ref() {
    assert!(has(here(), "secret_env_ref", "bad_value", "models[0].api_key"));
}
