//! Stable rule codes.

macro_rules! rules {
    ($($variant:ident => $code:literal),* $(,)?) => {
        /// Stable, finite rule codes: tests and tooling match on these, never on the message.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Rule { $($variant),* }
        impl Rule {
            pub fn code(self) -> &'static str {
                match self { $(Rule::$variant => $code),* }
            }
        }
    };
}

rules! {
    Parse => "parse",
    SpecVersion => "spec_version",
    BadValue => "bad_value",
    MissingField => "missing_field",
    UnexpectedField => "unexpected_field",
    ExactlyOne => "exactly_one",
    DuplicateId => "duplicate_id",
    UnknownRef => "unknown_ref",
    OutputNotText => "output_not_text",
    BatchNotAllowed => "batch_not_allowed",
    MaxTokensRequired => "max_tokens_required",
    MediaTwoWriters => "media_two_writers",
    MediaNotGenerated => "media_not_generated",
    MediaInputGenerated => "media_input_generated",
    MediaNotFromNode => "media_not_from_node",
    MediaNotMade => "media_not_made",
    MediaNeedsUtility => "media_needs_utility",
    MediaUtilityPath => "media_utility_path",
    MediaProviderFamily => "media_provider_family",
    MediaProviderConflict => "media_provider_conflict",
    MediaTypeUnsupported => "media_type_unsupported",
    PlaceholderMalformed => "placeholder_malformed",
    PlaceholderUnbound => "placeholder_unbound",
    PlaceholderUnused => "placeholder_unused",
    PlaceholderInputMixed => "placeholder_input_mixed",
    PromptRequired => "prompt_required",
    CleanupWrite => "cleanup_write",
    EdgeStart => "edge_start",
    EdgeEnd => "edge_end",
    BlockShape => "block_shape",
    BlockContext => "block_context",
    NetworkNotAllowed => "network_not_allowed",
    NotSupported => "not_supported",
    Io => "io",
    EnvMissing => "env_missing",
    EnvNotAllowed => "env_not_allowed",
    BlackboardHeader => "blackboard_header",
    BatchModels => "batch_models",
    BatchNested => "batch_nested",
    ReactNested => "react_nested",
    GroupOutRequired => "group_out_required",
    ValidationNoNext => "validation_no_next",
    ValidationInGroup => "validation_in_group",
    ModelFailed => "model_failed",
    ReactLimit => "react_limit",
    ToolLimit => "tool_limit",
    UtilityFailed => "utility_failed",
    TooManyRuns => "too_many_runs",
    RunTimeout => "run_timeout",
    ServiceStopped => "service_stopped",
    Interrupted => "interrupted",
    SessionFull => "session_full",
}
