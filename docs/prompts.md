# Prompts

A prompt is the text a node sends to its model. It is declared in `resources.prompts`, with an `id`, and a node uses
it with `resources.prompt`.

```yaml
resources:
  prompts:
    - id: analyst
      content:
        system: |
          You are an analyst specialized in contracts.
        user: |
          Context: {{findings}}
          Question: {{user_message}}
```

A prompt has exactly one of `content` (the texts written in the file) and `source` (`exactly_one`). A `source` with a
`path` is a `.yml` file with the same `content: { system, user }`; it is not Markdown, because one Markdown text could
not be split into a system text and a user text. It is read when the file is checked.

A `source` with a `url` answers the same text as such a file. It is not read when the file is checked: it is
downloaded when the run starts, before the first node, with the media read from a url (see
[media](media.md#a-media-from-a-url)), and its placeholders are checked then, with the rules below; the run stops at
the first one that is broken, while the check of the file reports all of them for a prompt it reads. A text that is not
`content: { system, user }` is `bad_value`; a text with a YAML alias or a tag, or with more than 100000 YAML events, is
`not_supported`, since it comes from outside the graph. Any of them stops the run before the first node, with the
error on the `url`.

```yaml
- id: reviewer
  source:
    url: https://prompts.example.com/reviewer.yml
    headers:
      Authorization: { env: PROMPTS_TOKEN }
```

## Placeholders

A placeholder is `{{name}}`: the name of something the node has, written between double braces. Before the call the
node replaces every placeholder with its value, in `system` and in `user`. It is a plain substitution: there is no
logic (no conditions, no loops), because the order of the work belongs to the graph, not to a text. A fixed value needs
no placeholder: it is written in the text.

A name is made of letters, digits, `_` and `-`; spaces inside the braces are ignored. A `{{` that is not closed, or an
invalid name, is an error (`placeholder_malformed`).

These are the names a prompt can use:

| Placeholder | Its value | Available when the node has |
|---|---|---|
| `{{user_message}}` | the message of the user | `message.input.user_message: true` |
| `{{agent_message}}` | the message of the node connected before | `message.input.agent_message: true` |
| `{{input_message}}` | the message, whichever its origin | exactly one of the two above |
| `{{chunks}}` | the text found in the vector databases of the node | `external.vectordb` |
| `{{<search-id>}}` | what the search engine finds (search cannot run yet) | that id in `external.search` |
| `{{<memory-id>}}` | the summary of the memory, empty until there is one | that id in `external.memory`, for a memory with `synt` |
| `{{<blackboard-id>}}` | the text of the blackboard | `blackboard` with `read: true` |

`{{input_message}}` is for a prompt shared by nodes whose message comes from different places. It cannot be used
together with `{{user_message}}` or `{{agent_message}}` (`placeholder_input_mixed`).

`search` cannot run yet: a node that uses it fails when it runs.

## The rules

The prompt and the node must agree in both directions. These rules are checked when the file is checked:

- every placeholder of the prompt must be available on the node (`placeholder_unbound`);
- everything the node declares must be used by the prompt (`placeholder_unused`): every search, every memory with
  `synt`, a blackboard it reads, and `{{chunks}}` when it reads a vector database;
- the prompt must use the message of the node: with both messages enabled it must use both `{{user_message}}` and
  `{{agent_message}}`, with one it must use it or `{{input_message}}` (`placeholder_unused`);
- `chunks`, `user_message`, `agent_message` and `input_message` are reserved names: nothing on the node can have one
  as its id, and the same id cannot appear twice on a node (`duplicate_id`).

## Without a prompt

A node may have no prompt. Then the model receives no system text, and its user text is the message of the node. This
needs exactly one of the two messages: with both, the model could not tell them apart (`prompt_required`). A node that
reads a vector database needs a prompt, since `{{chunks}}` has nowhere to go (`placeholder_unused`).

## `{{chunks}}`

For every vector database of the node, in the order the node lists them, the query is turned into a vector and
searched. The chunks found are joined one after the other with an empty line between them, the nearest first inside
each database, and the whole text is the value of `{{chunks}}`. The query is the message of the user
(`query: user_message`) or a fixed text written in the file; see [the reference](graph.md) for the `utility` that can
change the query and the chunks.

## Memory prompts

The prompt of a memory summary (`external.memory[].synt.prompt`) has its own two placeholders, `{{synt}}` and
`{{chat}}`, and must use both. See [memory](memory.md).
