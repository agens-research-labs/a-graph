# The command line

```
a-graph graph.yml "the message"                # checks the file, then runs it
cat question.txt | a-graph graph.yml           # without a message, the standard input is the message
a-graph graph.yml "the message" --chat_mode    # only `node: text`, one line for each result
a-graph --check graph.yml                      # only checks the file: nothing runs
a-graph --serve service.yml                    # starts the HTTP service
a-graph --print-schema                         # the JSON Schema of the graph file
a-graph --print-docs                           # the reference of the graph file, in Markdown
a-graph --print-service-schema                 # the JSON Schema of the file of the service
a-graph --print-service-docs                   # the reference of the file of the service
a-graph --print-service-api                    # the OpenAPI description of the HTTP service
a-graph --help                                 # the options
a-graph --version                              # the version
```

## The service

`--serve` starts the HTTP service instead of running a graph: see [the HTTP service](serve.md).

## The schema and the reference

`--print-schema` writes the JSON Schema (2020-12) of the graph file, the same as `spec/graph.schema.json`, and
`--print-docs` writes [the reference](graph.md). `--print-service-schema` and `--print-service-docs` do the same for
the file of the service: `spec/service.schema.json` and [its reference](service.md). `--print-service-api` writes the
OpenAPI 3.1 description of the HTTP service, `spec/service.openapi.json`. All five are generated from the code, take
no file and end with 0. A schema checks the shape of a file, the first step of `--check`; the other rules
are written in the descriptions of the keys.

## The message

The message of the user is the second argument. Without it, the standard input is read to its end and the line
breaks at its end are removed. Without a message and with no standard input to read (a terminal), the command stops
with exit code 2.

## Paths

Every path written in the graph (the `path` of a prompt, a media, a skill, a memory, a blackboard file, a log file)
is relative to the folder **the command is run from**, not to the folder of the graph.

## What goes where

Only the result goes to the standard output. Errors, the log (by default, see
[observability](observability.md)) and the display of `verbose` go to the standard error. A program that reads the
result can therefore read the standard output alone.

## The result

By default the result is one JSON document:

```json
{"items":[{"node":"assistant","message":"A mutex is a lock ...","media":[],"reasoning":[],"logprobs":[]}]}
```

`items` holds, in the order they were produced, the results that nodes send to the final output (see
[nodes](nodes.md)):

- `node`: the id of the node;
- `message`: the text of the node, a JSON value when the node has an output `schema`, or `null` when the node gave
  no text or does not send it (`assistant_message: false`);
- `media`: the media the node sent to the final output, each as `{ "id", "mime_type", "data" }`, with `data` in
  base64;
- `reasoning`: the reasoning of the model, when the node asks to show it;
- `logprobs`: when the node asks them, the tokens of the answer with their log probabilities, one list for each call
  to the model, in order, the last being the final answer; each token is
  `{ "token", "logprob", "bytes", "top_logprobs": [ { "token", "logprob", "bytes" } ] }` (see
  [nodes](nodes.md#where-the-answer-goes)).

With `--chat_mode` the result is written for a person instead: one `node: text` for each item that has a message, in
order. A text on several lines is written as it is, a JSON value on one line. Media, reasoning and logprobs are not
written.

## Errors

Every error is written on one line of the standard error, in this form:

```
[phase] path (rule): message
```

- `phase`: `validation` (the shape of the file and the links between ids), `interpretation` (the rules of each
  block, the environment variables, the hosts) or `execution` (the run);
- `path`: where in the file, for example `models[0].api_key`;
- `rule`: a stable code, for example `missing_field`;
- `message`: the explanation.

A file is checked as a whole before anything runs, and all its errors are written together, not only the first.

## Stopping a run

Ctrl-C (`SIGINT`) during a run stops it as a run that lasted too long is stopped: it writes `a-graph: stopping the run,
Ctrl-C again to quit at once`, its MCP servers are closed, its batch jobs still going on are cancelled at the provider
(see [models](models.md#batch)), its last event is logged and its trace is sent. Then it ends with the error
`interrupted` and exit code 3, in a few seconds at most. A second Ctrl-C ends the process at once, with exit code 3,
without waiting for any of that.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | the file is valid and, if it was run, the run ended well |
| 1 | the file is not valid (the errors are on the standard error) |
| 2 | wrong use: no file, a file that cannot be read, an unknown option, no message |
| 3 | the run failed: a model, a tool, a utility..., or it was stopped with Ctrl-C |
