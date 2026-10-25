# Utilities

A utility is a program or a service of yours that the **program** calls on the data of a node, before or after the
model: to convert a PDF into Markdown, to hide personal data, to add a mark. Unlike a [tool](tools.md), the model does
not know that utilities exist. They can be written in any language. Utilities are declared in `utilities`, each with an
`id`; a node names them where it wants its data changed. Every key is described in [the reference](graph.md).

```yaml
utilities:
  - id: pdf_to_md
    transport: stdio
    command: python3
    args: [utilities/pdf_to_md.py]
    timeout: 60
  - id: redact
    transport: http
    url: http://localhost:9000/redact
    headers: { Authorization: { env: REDACT_TOKEN } }
    timeout: 60
```

- `transport: stdio`: the program is started for each call. A container that runs once is a program too:
  `command: docker` with `args: [run, --rm, -i, <image>]`.
- `transport: http`: a service already running, which a-graph does not start or stop. Its host must be in the
  [allowed hosts](file.md#network).
- `timeout` (mandatory, at least 1): the seconds a call may take; after that the program is stopped or the request is
  abandoned.

## The contract

A utility receives one JSON object and answers one, on its standard input and output (`stdio`) or as the body of a
`POST` and of its answer (`http`).

It receives a text, or a file by its path:

```json
{ "text": "..." }
{ "path": "docs/a.pdf", "mime_type": "application/pdf" }
```

`mime_type` comes from the extension of the file (pdf, docx, xlsx, pptx, html, md, txt, csv, json, png, jpg, webp,
gif, mp3, wav, mp4), and is left out when the extension does not say it.

It answers a text, or a file in base64 with its type:

```json
{ "text": "..." }
{ "mime_type": "application/pdf", "data": "<base64>" }
```

## When a utility fails

A utility that fails stops the run, with its own message: the text it wrote on its standard error, or the status and
the body of its answer. No model is involved to read the reason. A utility that answers neither `text` nor
`mime_type` with `data`, or that does not answer within `timeout`, fails too (`utility_failed`).

## Where a node uses them

The key `utility` names one or more utilities, by id, in the order they run: each gets what the one before answered.
One id alone is a list of one.

| Place | What goes through | What they must give back |
|---|---|---|
| `message.input.utility` | the incoming text, from the user and from the node before | a text |
| `message.output.utility` | the text the node produces, once, before every destination | a text |
| `media.input[].utility` | a media the node reads; the first gets the path of the file | a text or an image |
| `media.output[].utility` | with `from: model`, the file the model made; with `from: message`, the text of the node | a file |
| `external.vectordb[].utility.query` | the query, before it is turned into a vector and searched | a text |
| `external.vectordb[].utility.chunks` | each chunk found, before the prompt | a text |
| `external.memory[].read.utility` | each message read back, and the summary | a text |
| `external.memory[].write.utility` | what is saved, after the utilities of `message.output` | a text |
| `blackboard.write.utility` | the text written, after the utilities of `message.output` | a text |

A utility that gives back a file where a text is needed fails the run.
