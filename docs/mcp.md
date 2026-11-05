# MCP servers

An MCP server is a server that speaks the [Model Context Protocol](https://modelcontextprotocol.io), the standard
protocol for existing servers. Its tools are given to the model next to the node's own [tools](tools.md), which use
the smaller contract of a-graph instead. Servers are declared in `resources.mcp_servers` and a node lists the ones it
uses in `resources.mcp_servers`. Every key is described in [the reference](graph.md).

```yaml
resources:
  mcp_servers:
    - id: files
      transport: stdio
      command: python3
      args: [mcp_servers/files.py]
      env: { ROOT: { env: FILES_ROOT } }
      max_iter: 10
      timeout: 30
    - id: search
      transport: http
      url: https://mcp.example.com/mcp
      headers: { Authorization: { env: MCP_TOKEN } }
      max_iter: 10
      timeout: 30
```

- `transport: stdio`: the program is started with `command` and `args`, and its `env` holds variables whose values
  come from the environment.
- `transport: http`: the server is reached at `url` with Streamable HTTP, the current transport of the protocol.

Only the **tools** of a server are used, never its resources or prompts.

## Connection

The protocol is spoken by `rmcp`, the official Rust SDK. It tries the current revision of MCP (`server/discover`,
without a handshake) and falls back to the older ones (`initialize`).

- The servers are connected once, when the run starts, and only the ones some node uses. Checking a file never
  connects them.
- At that moment each server is asked for its tools, which are then given to the model of every node that uses the
  server.
- A server that cannot be reached, or does not give its tools, stops the run before the first node.
- An answer of a server over HTTP is read up to 16 MiB: a server that answers with more when the run starts stops
  the run before the first node.
- Two tools with the same name on one node (from `tools`, from different servers, or a reserved name of the
  [skills](skills.md)) stop the run before the first node (`duplicate_id`): the model could not tell them apart.
- The connections stay open until the run ends, and are closed after the last node or after an error. A branch of a
  `parallel` or `concurrent` group stopped in the middle of a call lets go of the servers a moment later: the close
  waits for it, one second at most. A server still busy is let go after two seconds.

## Calls

- The result of a call is the structured content of the answer, if it has one; otherwise its text blocks, one after
  the other on separate lines.
- The arguments of a call are checked against the input schema the server gave for the tool, as for the `parameters`
  of [tools](tools.md): arguments that do not match are told to the model, and the tool is not called. A schema of the
  server that is not a valid JSON Schema is not enforced: the arguments of that tool are not checked, and the run says
  so once, with the warning `tool_schema_ignored`, when the servers are connected.
- An answer that the server marks as an error reaches the model as an error.
- A server that goes away during a call is connected again once, and the call is repeated. If that fails too, the
  model is told that the server is not reachable.
- A call that does not end within `timeout` seconds is a failure the model is told about.
- `max_iter` is the most calls to the tools of the server, any of them, in one run of a node. Asking once more fails
  the run (`tool_limit`), as for [tools](tools.md#the-loop).
