use super::*;

/// A plain call sends the headers and maps the answer.
#[tokio::test]
async fn plain_call() {
    let (r, req) = one_call(call("hi"), Resp::ok(text_answer("hello"))).await;
    let r = r.unwrap();
    assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/v1/messages"));
    assert!(req.headers.contains("x-api-key: test-key") && req.headers.contains("anthropic-version: 2023-06-01"), "{}", req.headers);

    assert_eq!(r.message.as_deref(), Some("hello"));
    assert!(r.structured.is_none() && r.tools_call.is_empty() && r.media.is_empty());
    assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (StopReason::End, Some("end_turn")));
    assert_eq!((r.info.provider.as_str(), r.info.model.as_str()), ("anthropic", "claude-haiku-4-5-20251001"));
    assert!(r.info.execution_time > 0.0);
    // a null count is 0; thinking tokens are informational
    assert_eq!(r.info.usage, Usage { input_tokens: 12, cache_read_tokens: 3, cache_write_tokens: 0, output_tokens: 7, reasoning_tokens: 2 });
}
