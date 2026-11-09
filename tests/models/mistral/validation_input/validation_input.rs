use super::*;

/// A validation error of Mistral lists what was sent (`detail[].input`): it stays out of the message.
#[tokio::test]
async fn validation_input() {
    let body = json!({ "detail": [{
        "loc": ["body", "messages", 0, "content"],
        "msg": "Input should be a valid list",
        "type": "list_type",
        "input": "the patient has diabetes"
    }] })
    .to_string();
    let (r, _) = one_call(call("q"), Resp::status(422, body)).await;
    let e = r.unwrap_err();
    assert_eq!(
        e.to_string(),
        "model `mistral_small` (mistral): the provider answered 422",
        "{e:?}"
    );
    assert!(
        e.said().is_some_and(|s| s.contains("the patient has diabetes")),
        "{e:?}"
    );
}
