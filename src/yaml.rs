//! Small YAML helpers shared by the blocks.

/// The most events of the parser a yml may have. The parse that follows keeps every event, then builds a tree from
/// them, at about 200 bytes an element: a text of 64 MiB could hold tens of millions of them, and take gigabytes. The
/// largest graph of the repository has under 800.
pub(crate) const MAX_YML_EVENTS: usize = 100_000;

/// What a yml from outside may not have: an alias, a tag, or more than `MAX_YML_EVENTS` events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Found {
    /// `*name`: serde_norway copies the value of the anchor at every alias, so a small yml can become a tree of
    /// gigabytes. Its own limit counts the aliases, not what they copy: one anchor of n values repeated by n aliases
    /// passes it and builds n² values.
    Alias,
    /// `!name` before a value: the service looks for a text or a list and skips a tagged value, while the build reads
    /// the value under the tag, so a tagged path or list of hosts would escape its rules.
    Tag,
    /// More events than `MAX_YML_EVENTS`: the reading stops there, so its time is bounded too.
    TooManyEvents,
}

/// The first alias or tag of the yml, or the event past `MAX_YML_EVENTS`. Only the events of the parser are read, one
/// at a time, and nothing is copied. A yml that does not parse before then answers `None`: serde_norway then reports
/// the error.
pub(crate) fn first_refused(yml: &str) -> Option<Found> {
    use std::mem::MaybeUninit;
    use unsafe_libyaml_norway::{
        YAML_ALIAS_EVENT, YAML_MAPPING_START_EVENT, YAML_SCALAR_EVENT, YAML_SEQUENCE_START_EVENT,
        YAML_STREAM_END_EVENT, YAML_UTF8_ENCODING, yaml_event_delete, yaml_event_t, yaml_parser_delete,
        yaml_parser_initialize, yaml_parser_parse, yaml_parser_set_encoding, yaml_parser_set_input_string,
        yaml_parser_t,
    };

    let mut parser = MaybeUninit::<yaml_parser_t>::uninit();
    let parser = parser.as_mut_ptr();
    // SAFETY: the parser is initialized before use and deleted once; it reads `yml`, which outlives it. Every event
    // is deleted after it is read; of its data, only the tag of a scalar, a sequence or a mapping is read, the field
    // of the union that belongs to that type, and only whether it is null.
    unsafe {
        if yaml_parser_initialize(parser).fail {
            return None;
        }
        yaml_parser_set_encoding(parser, YAML_UTF8_ENCODING);
        yaml_parser_set_input_string(parser, yml.as_ptr(), yml.len() as _);
        let mut found = None;
        let mut events = 0usize;
        loop {
            let mut event = MaybeUninit::<yaml_event_t>::uninit();
            if yaml_parser_parse(parser, event.as_mut_ptr()).fail {
                break;
            }
            let read = &*event.as_ptr();
            let kind = read.type_;
            let tagged = match kind {
                YAML_SCALAR_EVENT => !read.data.scalar.tag.is_null(),
                YAML_SEQUENCE_START_EVENT => !read.data.sequence_start.tag.is_null(),
                YAML_MAPPING_START_EVENT => !read.data.mapping_start.tag.is_null(),
                _ => false,
            };
            yaml_event_delete(event.as_mut_ptr());
            if kind == YAML_ALIAS_EVENT {
                found = Some(Found::Alias);
                break;
            }
            if tagged {
                found = Some(Found::Tag);
                break;
            }
            events += 1;
            if events > MAX_YML_EVENTS {
                found = Some(Found::TooManyEvents);
                break;
            }
            if kind == YAML_STREAM_END_EVENT {
                break;
            }
        }
        yaml_parser_delete(parser);
        found
    }
}
