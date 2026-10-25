# Memory

A memory keeps the conversation of a node between runs: what the user asked, what the node answered, and a summary of
what came before. Memories are declared in `external.memory`; a node uses one with `external.memory` on the node, where
it says which roles it reads and which it saves. Every key is described in [the reference](graph.md).

```yaml
external:
  memory:
    - id: session
      provider: json_file
      path: ./sessions/session_a.json
      synt: { length: 10, model: fast }
      chat: { length: 10 }

graph:
  nodes:
    - id: assistant
      external:
        memory:
          - id: session
            read: { user: true, assistant: true }
            write: { user: true, assistant: true }
```

Only `provider: json_file` runs today. `provider: vectordb` (with `ref`, the id of a vector database) is accepted when
the file is checked, but a node that uses it fails when it runs. A `json_file` memory needs `path` and `chat`.

## The file

The file is a JSON list of records, one for each node. A node reads and writes only its own record.

```json
[
  {
    "id": "assistant",
    "timestamp": "2026-10-13_21:12:00",
    "synt": "",
    "chat": [
      { "role": "user", "content": "...", "embedding": [0.1, 0.2] },
      { "role": "assistant", "content": "..." }
    ]
  }
]
```

- `id`: the node.
- `timestamp`: when the record was last changed, in UTC.
- `synt`: the summary, empty until there is one.
- `chat`: the messages not summarized yet, in the standard form of chat messages (`role` and `content`). A message of
  the user carries the vector of its text when the memory has `chat.embedding`.

A file that does not exist yet, an empty file, or a file without a record of the node, is an empty memory. The file,
its folder and the record are created at the first write. The file is written in one step (a temporary file, then
renamed).

## Reading

Before the call, for every memory of the node:

- `chat` becomes the history of the call: the messages that come before the new message. It holds the last
  `chat.length` exchanges (a message of the user and the answer after it) or, with `chat.embedding`, the `chat.length`
  exchanges whose message of the user is most similar to the new message, kept in the order they were saved. Only the
  roles that `read` takes are kept.
- `synt`, for a memory that has `synt`, goes into the prompt where it writes `{{<memory-id>}}` (see
  [prompts](prompts.md)). A memory without `synt` has no placeholder.

The `read.utility` changes each message read and the summary.

## Writing

After the node answers, the exchange of this turn is saved in the record of the node: the message the node received
(the message of the user, or else the message of the node before) and its answer, only the roles that `write` allows.
Any combination is allowed, for example only the messages of the user; what comes back later is what was saved.

The `write.utility` changes what is saved, after the utilities of `message.output`; the vector of a saved message is
made from the changed text.

## The summary

With `synt`, a memory is summarized when the record of the node reaches `synt.length` exchanges. Without `synt`, `chat`
only grows.

The model `synt.model` gets the previous summary and the exchanges to summarize, writes the new summary in at most
`synt.max_tokens` tokens, and the summarized exchanges leave `chat`. The call sends no `reasoning` setting, so the
model follows the default of its provider: a model that reasons spends part of `max_tokens` before it writes, which is
why the default is 4000. A summary with no text fails the run.

Without `synt.prompt` the program uses a prompt of its own. A prompt written for the summary must use both of its
placeholders (`placeholder_unused`):

- `{{synt}}`: the previous summary, empty the first time;
- `{{chat}}`: the exchanges to summarize, one message per line, as `user: ...` and `assistant: ...`.
