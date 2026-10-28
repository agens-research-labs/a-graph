//! Small URL helpers shared by the blocks, and the HTTP client they all start from.

/// The builder of every HTTP client. Redirects are not followed: the host of a redirect is not
/// checked against `allowed_hosts`, so a 3xx is an answer like any other non-2xx.
pub(crate) fn client() -> reqwest::ClientBuilder {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
}

/// Lowercase host of `scheme://[user@]host[:port][/...]`.
pub(crate) fn host_of(url: &str) -> Option<String> {
    host_port_of(url).map(|(host, _)| host)
}

/// Lowercase host and port of `scheme://[user@]host[:port][/...]`. The port is the one written,
/// or the one of the scheme (`http` 80, `https` 443); `None` for a scheme without a known port.
pub(crate) fn host_port_of(url: &str) -> Option<(String, Option<u16>)> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let (host, port) = if host_port.starts_with('[') {
        let (h, after) = host_port.split_once(']')?;
        (format!("{h}]"), after.strip_prefix(':'))
    } else {
        match host_port.split_once(':') {
            Some((h, p)) => (h.to_string(), Some(p)),
            None => (host_port.to_string(), None),
        }
    };
    if host.is_empty() {
        return None;
    }
    let port = match port {
        Some(p) if !p.is_empty() => p.parse().ok(),
        _ => match scheme.to_lowercase().as_str() {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        },
    };
    Some((host.to_lowercase(), port))
}

/// An entry of `allowed_hosts`, `host` or `host:port` (`[::1]:8080` for IPv6): its lowercase host
/// and its port. `Err` says what is wrong.
pub(crate) fn allowed_entry(entry: &str) -> Result<(String, Option<u16>), String> {
    let entry = entry.to_lowercase();
    let (host, port) = if entry.starts_with('[') {
        let (h, after) = entry.split_once(']').ok_or_else(|| format!("`{entry}` opens a `[` it does not close"))?;
        match after {
            "" => (format!("{h}]"), None),
            _ => (format!("{h}]"), Some(after.strip_prefix(':').ok_or_else(|| format!("`{entry}`: after `]` only `:<port>` may follow"))?)),
        }
    } else {
        match entry.split_once(':') {
            Some(_) if entry.matches(':').count() > 1 => return Err(format!("`{entry}`: an IPv6 address is written in brackets, as `[::1]` or `[::1]:8080`")),
            Some((h, p)) => (h.to_string(), Some(p)),
            None => (entry.clone(), None),
        }
    };
    if host.is_empty() || host == "[]" {
        return Err(format!("`{entry}` has no host"));
    }
    let port = match port {
        None => None,
        Some(p) => match p.parse::<u16>() {
            Ok(n) if n > 0 => Some(n),
            _ => return Err(format!("`{entry}`: the port must be a number from 1 to 65535")),
        },
    };
    Ok((host, port))
}

/// The text of an entry of `allowed_hosts`: `host` or `host:port`.
pub(crate) fn entry_text(host: &str, port: Option<u16>) -> String {
    match port {
        Some(p) => format!("{host}:{p}"),
        None => host.to_string(),
    }
}
