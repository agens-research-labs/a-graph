//! Small URL helpers shared by the blocks, and the HTTP client they all start from.

/// The builder of every HTTP client. Redirects are not followed: the host of a redirect is not
/// checked against `allowed_hosts`, so a 3xx is an answer like any other non-2xx.
pub(crate) fn client() -> reqwest::ClientBuilder {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
}

/// Lowercase host of `scheme://[user@]host[:port][/...]`.
pub(crate) fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = if host_port.starts_with('[') {
        host_port.split_once(']').map(|(h, _)| format!("{h}]"))?
    } else {
        host_port.split(':').next()?.to_string()
    };
    if host.is_empty() { None } else { Some(host.to_lowercase()) }
}
