use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An entry `host:port` allows that port only; the port of a URL is the written one or the one of
/// its scheme, and a provider host is reached on 443. An entry without a port allows every port.
/// An entry is read as the host of a URL is: `127.1` and `[0:0:0:0:0:0:0:1]` are `127.0.0.1` and `[::1]`.
#[test]
fn host_ports() {
    assert_eq!(errors(here(), "port_allowed"), vec![]);
    assert_eq!(errors(here(), "any_port"), vec![]);
    assert_eq!(errors(here(), "ipv6"), vec![]);
    assert_eq!(errors(here(), "normalized"), vec![]);
    for path in ["models[0]", "models[1]", "trace.endpoint"] {
        assert!(refused(here(), "port_refused", path), "{path}");
    }
}

/// A port that is not a number from 1 to 65535, an IPv6 address without brackets, a URL or a host
/// with a path is a bad value.
#[test]
fn host_ports_bad() {
    let mut doc = load(&yml_in(here(), "bad_port")).unwrap_or_else(|e| panic!("{e}"));
    let errs = allowed_hosts(doc.take(key::PASSPORT)).unwrap_err();
    let got: Vec<(String, String)> = errs
        .0
        .iter()
        .map(|e| (e.rule.code().to_string(), e.path.to_string()))
        .collect();
    let at = |i: usize| {
        (
            "bad_value".to_string(),
            format!("passport.adl.permissions.network.allowed_hosts[{i}]"),
        )
    };
    assert_eq!(got, vec![at(0), at(1), at(2), at(3), at(4), at(5)]);
}
