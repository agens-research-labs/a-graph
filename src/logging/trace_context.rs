//! The W3C Trace Context of a request to the service: the `traceparent` header a caller sends to put the run inside
//! its own trace, and the one the answer gives back with the span of the run
//! (<https://www.w3.org/TR/trace-context/>).

/// The name of the header, in the request and in the answer.
pub const TRACEPARENT: &str = "traceparent";

/// The trace of the caller, from a valid `traceparent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    /// 32 lowercase hex digits, not all zero.
    pub trace_id: String,
    /// The span of the caller, parent of the run; never 0.
    pub parent: u64,
    /// The flag `sampled`: whether the caller records the trace.
    pub sampled: bool,
}

impl TraceContext {
    /// The trace of the caller, when the request has exactly one `traceparent` and it is valid; otherwise `None`, as if
    /// there were no header. Two headers are refused: nothing tells which one is right.
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Self> {
        let mut all = headers.get_all(TRACEPARENT).iter();
        match (all.next(), all.next()) {
            (Some(one), None) => Self::parse(one.to_str().ok()?),
            _ => None,
        }
    }

    /// Reads `version-trace_id-parent-flags`, all lowercase hex. The version `00` is exactly these four fields
    /// (§3.2.2); a later version may add fields after a dash, which are not read (§3.2.4); `ff` is forbidden. Ids of
    /// all zeros are invalid, in every version.
    fn parse(value: &str) -> Option<Self> {
        let b = value.as_bytes();
        if b.len() < 55 || b[2] != b'-' || b[35] != b'-' || b[52] != b'-' {
            return None;
        }
        let hex = |s: &[u8]| s.iter().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'));
        let zero = |s: &[u8]| s.iter().all(|c| *c == b'0');
        let (version, trace, parent, flags) = (&b[0..2], &b[3..35], &b[36..52], &b[53..55]);
        if !hex(version) || version == b"ff" || !hex(trace) || !hex(parent) || !hex(flags) {
            return None;
        }
        match (version == b"00", b.get(55)) {
            (_, None) | (false, Some(b'-')) => {}
            _ => return None,
        }
        if zero(trace) || zero(parent) {
            return None;
        }
        // the fields are ASCII hex here, so they are text and numbers
        let text = |s: &[u8]| String::from_utf8_lossy(s).into_owned();
        Some(TraceContext {
            trace_id: text(trace),
            parent: u64::from_str_radix(&text(parent), 16).ok()?,
            sampled: u8::from_str_radix(&text(flags), 16).ok()? & 1 == 1,
        })
    }

    /// The `traceparent` of a run, version `00`: its trace, its span, and `01` when its trace is sent, `00` otherwise.
    pub fn traceparent(trace_id: &str, span: u64, sent: bool) -> String {
        format!("00-{trace_id}-{span:016x}-{:02x}", u8::from(sent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";

    fn headers(values: &[&str]) -> reqwest::header::HeaderMap {
        let mut map = reqwest::header::HeaderMap::new();
        for v in values {
            map.append(TRACEPARENT, v.parse().unwrap());
        }
        map
    }

    #[test]
    fn valid_header_read() {
        let c = TraceContext::from_headers(&headers(&[&format!("00-{TRACE}-00f067aa0ba902b7-01")])).unwrap();
        assert_eq!(c.trace_id, TRACE);
        assert_eq!(c.parent, 0x00f067aa0ba902b7);
        assert!(c.sampled);
        let c = TraceContext::from_headers(&headers(&[&format!("00-{TRACE}-00f067aa0ba902b7-00")])).unwrap();
        assert!(!c.sampled, "flag 00: the caller does not record");
        let c = TraceContext::from_headers(&headers(&[&format!("00-{TRACE}-00f067aa0ba902b7-fe")])).unwrap();
        assert!(!c.sampled, "only the bit `sampled` is read");
    }

    #[test]
    fn name_in_any_case() {
        let mut map = reqwest::header::HeaderMap::new();
        map.append(
            reqwest::header::HeaderName::from_bytes(b"TraceParent").unwrap(),
            format!("00-{TRACE}-00f067aa0ba902b7-01").parse().unwrap(),
        );
        assert!(TraceContext::from_headers(&map).is_some());
    }

    #[test]
    fn later_version_read() {
        let c = TraceContext::from_headers(&headers(&[&format!("01-{TRACE}-00f067aa0ba902b7-01-abcd")])).unwrap();
        assert_eq!(c.trace_id, TRACE);
        assert!(TraceContext::from_headers(&headers(&[&format!("01-{TRACE}-00f067aa0ba902b7-01")])).is_some());
    }

    #[test]
    fn invalid_headers_ignored() {
        let bad = [
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01".to_string(),
            format!("00-{TRACE}-0000000000000000-01"),
            format!("00-{}-00f067aa0ba902b7-01", TRACE.to_uppercase()),
            format!("00-{TRACE}-00F067AA0BA902B7-01"),
            format!("00-{TRACE}-00f067aa0ba902b7-0G"),
            format!("ff-{TRACE}-00f067aa0ba902b7-01"),
            format!("0g-{TRACE}-00f067aa0ba902b7-01"),
            format!("00-{TRACE}-00f067aa0ba902b7-01-abcd"),
            format!("00-{TRACE}-00f067aa0ba902b7-01x"),
            format!("01-{TRACE}-00f067aa0ba902b7-01abcd"),
            format!("01-{}-00f067aa0ba902b7-01", TRACE.to_uppercase()),
            format!("00-{TRACE}-00f067aa0ba902b7-1"),
            format!("00_{TRACE}_00f067aa0ba902b7_01"),
            "hello".to_string(),
            String::new(),
        ];
        for value in &bad {
            assert_eq!(
                TraceContext::from_headers(&headers(&[value])),
                None,
                "accepted: {value}"
            );
        }
        assert_eq!(TraceContext::from_headers(&headers(&[])), None, "no header");
        let good = format!("00-{TRACE}-00f067aa0ba902b7-01");
        assert_eq!(
            TraceContext::from_headers(&headers(&[&good, &good])),
            None,
            "two headers"
        );
        let mut map = reqwest::header::HeaderMap::new();
        map.append(
            TRACEPARENT,
            reqwest::header::HeaderValue::from_bytes(format!("00-{TRACE}-00f067aa0ba902b\u{e9}-01").as_bytes())
                .unwrap(),
        );
        assert_eq!(TraceContext::from_headers(&map), None, "not ASCII");
    }

    #[test]
    fn traceparent_written() {
        assert_eq!(
            TraceContext::traceparent(TRACE, 0x9c41e07a3b5d2f18, true),
            format!("00-{TRACE}-9c41e07a3b5d2f18-01")
        );
        assert_eq!(
            TraceContext::traceparent(TRACE, 1, false),
            format!("00-{TRACE}-0000000000000001-00")
        );
    }
}
