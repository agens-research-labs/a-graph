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

/// Lowercase host and port of a URL, as the `url` crate reads it: reqwest reads it the same way, so
/// the host checked is the host called. The port is the one written, or the one of the scheme
/// (`http` 80, `https` 443); `None` for a scheme without a known port (`s3://`). An IPv6 host keeps
/// its brackets (`[::1]`). A URL with blanks or control characters is refused: the parser would drop them.
pub(crate) fn host_port_of(url: &str) -> Option<(String, Option<u16>)> {
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str().filter(|h| !h.is_empty())?;
    Some((host.to_lowercase(), parsed.port_or_known_default()))
}

/// An entry of `allowed_hosts`, `host` or `host:port` (`[::1]:8080` for IPv6): its host and its
/// port. The host is read by the `url` crate as the host of a URL is, so the two sides of the check
/// are written the same way: `127.1` is `127.0.0.1`, `[0:0:0:0:0:0:0:1]` is `[::1]`, a domain is
/// lowercase. `Err` says what is wrong.
pub(crate) fn allowed_entry(entry: &str) -> Result<(String, Option<u16>), String> {
    if entry.contains("://") {
        return Err(format!(
            "`{entry}` is a URL: the list takes host names, as `api.example.com` or `localhost:11434`"
        ));
    }
    let entry = entry.to_lowercase();
    let (host, port) = if entry.starts_with('[') {
        let (h, after) = entry
            .split_once(']')
            .ok_or_else(|| format!("`{entry}` opens a `[` it does not close"))?;
        match after {
            "" => (format!("{h}]"), None),
            _ => (
                format!("{h}]"),
                Some(
                    after
                        .strip_prefix(':')
                        .ok_or_else(|| format!("`{entry}`: after `]` only `:<port>` may follow"))?,
                ),
            ),
        }
    } else {
        match entry.split_once(':') {
            Some(_) if entry.matches(':').count() > 1 => {
                return Err(format!(
                    "`{entry}`: an IPv6 address is written in brackets, as `[::1]` or `[::1]:8080`"
                ));
            }
            Some((h, p)) => (h.to_string(), Some(p)),
            None => (entry.clone(), None),
        }
    };
    if host.is_empty() || host == "[]" {
        return Err(format!("`{entry}` has no host"));
    }
    let host = url::Host::parse(&host)
        .map_err(|e| format!("`{entry}`: `{host}` is not a host name ({e})"))?
        .to_string();
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
