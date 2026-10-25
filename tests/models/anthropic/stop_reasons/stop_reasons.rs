use super::*;

/// Stop reasons.
#[tokio::test]
async fn stop_reasons() {
    for (provider_value, expected) in [("model_context_window_exceeded", StopReason::ContextExceeded), ("refusal", StopReason::Refused), ("pause_turn", StopReason::Other), ("stop_sequence", StopReason::End)] {
        let (r, _) = one_call(call("q"), Resp::ok(message(json!([{ "type": "text", "text": "x" }]), provider_value))).await;
        let r = r.unwrap();
        assert_eq!(r.stop_reason, expected, "{provider_value}");
        assert_eq!(r.stop_detail.as_deref(), Some(provider_value));
    }
}
