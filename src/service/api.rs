//! The routes of the service and their description in OpenAPI 3.1: `spec/service.openapi.json`,
//! printed by `a-graph --print-service-api`. The server builds its routes from `Endpoint::ALL`,
//! so a route is described if and only if it exists.

use serde_json::{Value, json};

/// One route of the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    Health,
    Run,
    Start,
    State,
    Events,
}

impl Endpoint {
    pub const ALL: [Endpoint; 5] = [
        Endpoint::Health,
        Endpoint::Run,
        Endpoint::Start,
        Endpoint::State,
        Endpoint::Events,
    ];

    pub fn path(self) -> &'static str {
        match self {
            Endpoint::Health => "/health",
            Endpoint::Run => "/run",
            Endpoint::Start => "/runs",
            Endpoint::State => "/runs/{id}",
            Endpoint::Events => "/runs/{id}/events",
        }
    }

    /// `get` or `post`, as OpenAPI writes it.
    pub fn method(self) -> &'static str {
        match self {
            Endpoint::Run | Endpoint::Start => "post",
            Endpoint::Health | Endpoint::State | Endpoint::Events => "get",
        }
    }

    /// The operation of OpenAPI.
    fn operation(self) -> Value {
        let errors = |description: &str| json!({ "description": description, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Errors" } } } });
        let request = json!({ "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Request" } } } });
        let id = json!([{ "name": "id", "in": "path", "required": true, "description": "The id of the run, as `POST /runs` gave it.", "schema": { "type": "string" } }]);
        let caller = json!([{ "name": "traceparent", "in": "header", "required": false, "description": "The trace of the caller (W3C Trace Context): when it is valid, the run is in that trace, under the span it names, and with the flag `00` its trace is not sent. A header that is not valid, or two of them, are as no header: the run has a trace of its own.", "schema": { "type": "string" } }]);
        let traceparent = json!({ "traceparent": { "description": "The trace of the run and its span (W3C Trace Context, version `00`): the trace of the caller, or one of its own; the flag is `01` when the trace of the run is sent to a collector, `00` otherwise.", "schema": { "type": "string" } } });
        let refusals = |mut responses: serde_json::Map<String, Value>| {
            responses.insert("400".into(), errors("The body is not a JSON object with `yml`, `message` and, if wanted, `session` (an unknown key too), or `session` is not a name."));
            responses.insert(
                "408".into(),
                errors("The body did not arrive within `read_timeout_seconds` (`read_timeout`)."),
            );
            responses.insert(
                "413".into(),
                json!({ "description": "The body is larger than 64 MiB." }),
            );
            responses.insert(
                "422".into(),
                errors(
                    "The graph is not valid, it breaks a rule of the service, or the session is full (`session_full`).",
                ),
            );
            responses.insert(
                "500".into(),
                errors("The folder of the session cannot be created or weighed."),
            );
            responses.insert(
                "503".into(),
                errors(
                    "The service is running its most runs, `max_runs`, or this address already has its most requests whose body is still arriving, `max_reading_per_address` (`too_many_runs`): try again later.",
                ),
            );
            Value::Object(responses)
        };
        match self {
            Endpoint::Health => json!({
                "summary": "Whether the service is up.",
                "responses": { "200": { "description": "The service is up.", "content": { "application/json": { "schema": {
                    "type": "object", "properties": { "status": { "const": "ok" } }, "required": ["status"], "additionalProperties": false,
                } } } } },
            }),
            Endpoint::Run => {
                let mut ok = serde_json::Map::new();
                ok.insert("200".into(), json!({ "description": "The run ended well: its result, the same JSON as the command line writes.", "headers": traceparent.clone(), "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Result" } } } }));
                let mut failed = errors(
                    "The run failed: a model, a utility, a tool, or it lasted more than `run_timeout_seconds` (`run_timeout`), or the service was stopped before it ended (`service_stopped`).",
                );
                failed["headers"] = traceparent;
                ok.insert("502".into(), failed);
                json!({
                    "summary": "Runs a graph and answers when it is done.",
                    "description": "The graph is built again for each request: requests share nothing but the folders of their sessions.",
                    "parameters": caller,
                    "requestBody": request,
                    "responses": refusals(ok),
                })
            }
            Endpoint::Start => {
                let mut ok = serde_json::Map::new();
                ok.insert("202".into(), json!({ "description": "The run started. The refusals of `POST /run` happen here, before any run is created.", "headers": traceparent, "content": { "application/json": { "schema": {
                    "type": "object", "properties": { "run": { "type": "string", "description": "The id of the run, random: whoever knows it can watch the run." } }, "required": ["run"], "additionalProperties": false,
                } } } }));
                json!({
                    "summary": "Starts a graph and answers at once with the id of the run, to watch it while it goes.",
                    "parameters": caller,
                    "requestBody": request,
                    "responses": refusals(ok),
                })
            }
            Endpoint::State => json!({
                "summary": "The state of a run.",
                "parameters": id,
                "responses": {
                    "200": { "description": "The state of the run.", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/State" } } } },
                    "404": errors("There is no such run: it never existed, or it was forgotten (see `retention_seconds`)."),
                },
            }),
            Endpoint::Events => json!({
                "summary": "The events of a run, as Server-Sent Events.",
                "description": "The events already happened, then the new ones as they come, and at the end the event `result` (its data is the JSON of `POST /run`) or `error` (its data is `{ \"errors\": [...] }`); then the stream closes. Connecting again gives it all again. A run keeps its last 1000 events: a watcher that comes after older ones were dropped first gets the event `dropped`, `{ \"count\": n }`, with how many it missed. The events are those of the `verbose` of the graph, at its level: with `off` only the last one; with `steps` `run.start`, `run.end`, `node.start`, `node.end`, `group.start`, `group.end` and the warnings and errors; with `detail` also `model.call`, `tool.call`, `utility.call`, `read.vectordb`, `read.memory`, `read.media` and `read.prompt`. The data of an event is a JSON object with its fields and `node`; its contents only as the passport of the graph allows, otherwise `content_redacted: true`.",
                "parameters": id,
                "responses": {
                    "200": { "description": "The stream of the events.", "content": { "text/event-stream": { "schema": { "type": "string" } } } },
                    "404": errors("There is no such run: it never existed, or it was forgotten (see `retention_seconds`)."),
                },
            }),
        }
    }
}

/// The description of the API, as the text of `spec/service.openapi.json`.
pub fn openapi() -> String {
    let mut paths = serde_json::Map::new();
    for e in Endpoint::ALL {
        let item = paths.entry(e.path()).or_insert_with(|| json!({}));
        item[e.method()] = e.operation();
    }
    let doc = json!({
        "openapi": "3.1.0",
        "info": {
            "title": "a-graph service",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "The HTTP service of a-graph, `a-graph --serve service.yml`: it runs the graphs it is sent. Generated from the code by `a-graph --print-service-api`: do not edit it by hand.",
        },
        "paths": paths,
        "components": { "schemas": schemas() },
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"
}

fn schemas() -> Value {
    json!({
        "Request": {
            "type": "object",
            "properties": {
                "yml": { "type": "string", "description": "The text of the graph, sent every time." },
                "message": { "type": "string", "description": "The message of the user." },
                "session": {
                    "type": "string",
                    "pattern": "^(?!\\.+$)[A-Za-z0-9._-]{1,64}$",
                    "default": "default",
                    "description": "The session: the folder, inside `data_dir`, where the files of the graph are. Letters, digits, `-`, `_` and `.`, at most 64, not dots only.",
                },
            },
            "required": ["yml", "message"],
            "additionalProperties": false,
        },
        "Result": {
            "type": "object",
            "properties": { "items": { "type": "array", "items": { "$ref": "#/components/schemas/Item" }, "description": "What the nodes sent to the final output, in the order it was produced." } },
            "required": ["items"],
            "additionalProperties": false,
        },
        "Item": {
            "type": "object",
            "properties": {
                "node": { "type": "string", "description": "The id of the node." },
                "message": { "description": "The text of the node, a JSON value when the node has an output `schema`, or `null` when the node gave no text." },
                "media": { "type": "array", "items": { "$ref": "#/components/schemas/Media" }, "description": "The media the node sent to the final output." },
                "reasoning": { "type": "array", "description": "The reasoning of the model, when the node asks to show it." },
                "logprobs": { "type": "array", "items": { "type": "array", "items": { "$ref": "#/components/schemas/Token" } }, "description": "When the node asks them: the tokens of the answer with their log probabilities, one list for each call to the model, in order; the last is the final answer." },
            },
            "required": ["node", "message", "media", "reasoning", "logprobs"],
            "additionalProperties": false,
        },
        "Token": {
            "type": "object",
            "properties": {
                "token": { "type": "string" },
                "logprob": { "type": "number", "description": "The natural logarithm of the probability of the token." },
                "bytes": { "type": ["array", "null"], "items": { "type": "integer" }, "description": "The bytes of the token, when the provider gives them." },
                "top_logprobs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "token": { "type": "string" },
                            "logprob": { "type": "number" },
                            "bytes": { "type": ["array", "null"], "items": { "type": "integer" } },
                        },
                        "required": ["token", "logprob", "bytes"],
                        "additionalProperties": false,
                    },
                    "description": "The most likely tokens at the same place, as many as the node asks.",
                },
            },
            "required": ["token", "logprob", "bytes", "top_logprobs"],
            "additionalProperties": false,
        },
        "Media": {
            "type": "object",
            "properties": {
                "id": { "type": ["string", "null"], "description": "The id of the media in the graph." },
                "mime_type": { "type": "string" },
                "data": { "type": "string", "description": "The content, in base64." },
            },
            "required": ["id", "mime_type", "data"],
            "additionalProperties": false,
        },
        "Errors": {
            "type": "object",
            "properties": { "errors": { "type": "array", "items": { "$ref": "#/components/schemas/Error" } } },
            "required": ["errors"],
            "additionalProperties": false,
        },
        "Error": {
            "type": "object",
            "properties": {
                "phase": { "enum": ["validation", "interpretation", "execution"] },
                "path": { "type": "string", "description": "Where in the graph, for example `models[0].api_key`." },
                "rule": { "type": "string", "description": "A stable code, for example `missing_field`." },
                "message": { "type": "string" },
            },
            "required": ["phase", "path", "rule", "message"],
            "additionalProperties": false,
        },
        "State": {
            "oneOf": [
                { "type": "object", "properties": { "state": { "const": "running" } }, "required": ["state"], "additionalProperties": false },
                { "type": "object", "properties": { "state": { "const": "done" }, "result": { "$ref": "#/components/schemas/Result" } }, "required": ["state", "result"], "additionalProperties": false },
                { "type": "object", "properties": { "state": { "const": "failed" }, "errors": { "type": "array", "items": { "$ref": "#/components/schemas/Error" } } }, "required": ["state", "errors"], "additionalProperties": false },
            ],
        },
    })
}
