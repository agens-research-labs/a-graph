# Embeddings

An embedding model turns a text into a vector. Embedding models are declared in `embeddings`, each with an `id`; a
[vector database](vectordb.md) cites one with its `embedding`, and a [memory](memory.md) with `chat.embedding`. Every
key is described in [the reference](graph.md).

```yaml
embeddings:
  - id: embedder
    provider: ollama
    model: embeddinggemma
    host: http://localhost:11434
```

## Providers

`ollama` is the only provider: an Ollama server, called with `POST <host>/api/embed` and the body
`{ "model", "input" }`, where `input` is the list of the texts. It takes no credentials. The host of `host`, on its
port, must be in `allowed_hosts` (see [the file](file.md#network)).

## What is turned into a vector

- For a vector database: the query of the node, at each search (see [vector databases](vectordb.md#a-search)).
- For a memory with `chat.embedding`: the message the node takes, when the history is read, and again when it is
  saved (`write.user`), after the utilities of `write.utility` (see [memory](memory.md)).

## Errors

The server answers one vector for each text, in the same order. An answer that is not JSON, that has no `embeddings`
list, with a vector that is not a non-empty list of numbers, or with more or fewer vectors than texts, fails the node
(`model_failed`). So does an answer with an error, as for a model (see [models](models.md#errors)). A call that does
not end within 10 minutes is abandoned and fails like a network error.
