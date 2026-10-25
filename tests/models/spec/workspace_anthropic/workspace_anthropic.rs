use super::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Workspace id is for anthropic only.
#[test]
fn workspace_anthropic() {
    // the Loader refuses the field on any other provider
    assert!(load(&yml_in(here(), "workspace_anthropic")).is_err());
}
