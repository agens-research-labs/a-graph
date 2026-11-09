use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// More than the 16 MiB `rmcp` reads of one JSON answer of an MCP server over HTTP (RUSTSEC-2026-0462).
const PADDING: usize = 20 * 1024 * 1024;

/// An MCP server over HTTP that speaks the older revision (`initialize`) and has no tools; every JSON answer
/// begins with `PADDING` spaces, which a JSON parser skips.
fn padded_server(req: &Req) -> Resp {
    if req.method != "POST" {
        return Resp::status(405, "");
    }
    let msg: Value = serde_json::from_str(&req.body).unwrap();
    let Some(id) = msg.get("id").cloned() else {
        // a notification
        return Resp::status(202, "");
    };
    let answer = match msg["method"].as_str() {
        Some("initialize") => json!({ "jsonrpc": "2.0", "id": id, "result": {
            "protocolVersion": msg["params"]["protocolVersion"],
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "padded", "version": "1" },
        } }),
        Some("tools/list") => json!({ "jsonrpc": "2.0", "id": id, "result": { "tools": [] } }),
        _ => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "method not found" } }),
    };
    Resp::ok(format!("{}{answer}", " ".repeat(PADDING)))
}

/// The answer of an MCP server over HTTP is not read past its limit: a server that answers with more than 16 MiB
/// cannot be connected, and the run stops before the first node, though each answer is valid JSON.
#[tokio::test]
async fn http_body_limit() {
    let (server, asked) = listen(padded_server).await;
    let (model_port, seen) = listen(model(vec![], false)).await;
    let yml = yml_with(here(), "http_body_limit", &[("model", model_port), ("mcp", server)]);
    let p = Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}")));
    let e = p.run("go").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("io", "resources.mcp_servers".to_string())
    );
    assert!(e.message.contains("MCP server `srv`"), "{}", e.message);
    assert!(!asked.lock().unwrap().is_empty(), "the MCP server was not asked");
    assert!(seen.lock().unwrap().is_empty(), "the model was called");
}
