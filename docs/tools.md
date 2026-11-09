# Tools

A tool is a program or a service of yours that the model may call while a node runs. The model decides whether to call
it; the program makes the call. Tools are declared in `resources.tools` and a node lists the ones it uses in
`resources.tools`. Every key is described in [the reference](graph.md).

```yaml
resources:
  tools:
    - id: get_weather
      description: Returns the current weather for a city
      transport: stdio
      command: python3
      args: [tools/weather.py]
      max_iter: 10
      timeout: 30
      parameters:
        type: object
        properties:
          city: { type: string, description: City name }
        required: [city]
```

The model knows a tool by its `id`, its `description` and its `parameters`, a JSON Schema given to the provider as it
is written. `parameters` must be a valid JSON Schema: one that is not is an error of the file (`bad_value`).

The same mechanism serves the tools of [MCP servers](mcp.md), which speak a standard protocol, and the tools of
[skills](skills.md).

## The contract

The model never talks to a tool: every call goes through the program. A tool receives one JSON object, the arguments
the model chose, and answers one JSON value, the result. The contract is the same on both transports.

- `transport: stdio`: the program `command` is started, with `args`, for each call. The arguments arrive on its
  standard input; the result is what it writes on its standard output. An exit code other than 0 is a failure, and
  the model receives the text of the standard error.
- `transport: http`: the arguments are the body of a `POST` to `url`, with the `headers` of the tool; the result is
  the body of the answer. A status other than 2xx is a failure, and the model receives the status and the body.

An answer that is not JSON is a failure too.

## When a call goes wrong

- Arguments that do not match the `parameters` of the tool: the tool is not called, and the model is told the
  reasons, as for any other failure. The call counts in `max_iter`.
- A tool that fails, does not answer within `timeout` seconds, or answers something that is not JSON: the model is
  told why, and may try again or answer otherwise. A program that runs out of time is killed; a request is abandoned.
- A call of a tool the node does not have: the model is told that the node has no such tool.
- A program that cannot even start: the run stops. It is a mistake of the configuration, not something the model can
  fix.
- An answer of the model that stopped before its end, at `max_tokens` or at the end of the context window, and asks
  for tools: no tool is called, since a call may be cut anywhere, and the run stops (`model_failed`). It happens
  with or without an output `schema`; a larger `max_tokens` is the fix. Without a `schema`, a cut answer that asks
  for no tool is the answer of the node.

## The loop

1. The model is called with the tools of the node.
2. If it asks for tools, they are called. The calls asked in one answer run together.
3. The results go back to the model, which is called again.
4. When the model answers without asking for a tool, that answer is the answer of the node.

Each tool may be called at most `max_iter` times in one run of a node, timeouts and failures included. The count
starts again for each node, and for each appearance of the same node in the edges. Asking once more fails the run
(`tool_limit`). The tools of an MCP server and the files and scripts of a skill have their own caps, written on the
server and on the skill.

The rounds of the loop are limited too: the model may ask for tools in at most `resources.max_rounds` rounds of one run
of the node (20 by default, at least 1). Every round counts, whatever the model asks for: also a skill, or a tool the
node does not have, which no `max_iter` counts. One more round fails the run (`tool_limit`).

The calls are written in the log and in the display as `tool.call` events (see [observability](observability.md)).
