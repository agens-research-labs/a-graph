use crate::common::network::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The fixture reaches only allowed hosts.
#[test]
fn valid_hosts_ok() {
    assert_eq!(errors_of(&own_yml(here())), vec![]);
}
