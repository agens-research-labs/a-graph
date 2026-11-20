# Media

A media is everything that is not the text of a message: a document, an image, an audio. Media are declared in
`resources.media`, each with an `id` and a `type`; a node reads them with `media.input` and produces them with
`media.output`. Every key is described in [the reference](graph.md).

## Types

- `md`: a text. A file of another format, such as a PDF, is declared `md` too, and the node that reads it names a
  [utility](utilities.md) that changes it into text: `type` is what the model gets.
- `file`: a file of any format, made by a utility from the text of a node. The program does not look inside it: it
  delivers it in base64, with the type the utility says.
- `{ img: <format> }`, `{ audio: <format> }`, `{ video: <format> }`: an image, an audio or a video, with its format
  (`png`, `jpeg`, `wav`, `mp4`...).

## Three kinds of media

**A media that is read** has a `source`: exactly one of `path` (a file), `url` (see
[below](#a-media-from-a-url), with optional `headers` and `mime_type`) and `base64` (the content itself, with an
optional `mime_type`, its type, given to a utility).

```yaml
- id: contract
  type: md
  source: { path: ./docs/contract.md }
```

**A media produced by a model** has `generate` instead of `source`: how it is made.

```yaml
- id: cover
  type: { img: png }
  generate:
    size: { width: 1024, height: 1024 }
    quality: high
    background: true
  provider:
    openai: { moderation: low }
```

**A media made by a utility** from the text of a node has type `file`, and neither `source` nor `generate`.

A media has exactly one of `source` and `generate`, except one of type `file`, which has neither (`exactly_one`).

## Reading media

An entry of `media.input` says where the media comes from:

- `from: resources`: a media with a `source`. A media that a node produces cannot be read from the resources
  (`media_input_generated`).
- `from: node`: a media produced by a node connected before, which lists it in its `media.output`. If no node before
  produces it, the file is not valid (`media_not_from_node`). It arrives only if that entry has `node: true`;
  otherwise the node fails when it runs.

An entry may have a `utility`: the media goes through those utilities before the node uses it, a program by the path
of a file on disk and every other utility by its bytes (see [the contract](utilities.md#the-contract)); a-graph writes
no file for them. The utilities must end with a text or an image.

What a model can read:

- a text (`md`), and images in the formats its provider accepts (see [models](models.md#what-each-provider-can-do));
- not a media of type `file`: it needs a utility;
- not audio or video: they are not supported yet;
- a media from a node, without utilities, only if it is an image;
- a media from a `url`, as a media from a `path`: a `md` must be text.

### A media from a url

A media or a prompt with `source: { url }` is downloaded when the run starts, before the first node, and kept until
the run ends: each node that uses it reads the same bytes, downloaded once. Building the program, and `--check`, never
call it. Only the urls a node uses are downloaded, and the prompt of the summary of a memory a node uses; a url is
downloaded even when its node is never reached in that run. Each media and each prompt has its own download, even
when two of them name the same url.

```yaml
- id: report
  type: md
  source:
    url: https://files.example.com/report.md
    headers:
      Authorization: { env: FILES_TOKEN }
```

- `headers`: the headers of the request, each read from an environment variable when the file is checked, as the
  headers of a tool (`env_missing`; on [the service](serve.md) also `allowed_env`).
- `mime_type`: the type given to the utilities the media goes through. Without it, the type comes from the extension
  of the path of the url, as for a file on disk, the query left out: `report.pdf?sig=abc` is `application/pdf`.
- The host must be in `allowed_hosts`, checked with the file; the scheme must be `http` or `https`. Redirects are not
  followed.
- All the urls of a run together may weigh 64 MiB, as much as the body of a request to the service, and take 60
  seconds. Neither can be changed. The bytes are counted as they arrive, after decompression: an answer compressed
  with gzip or deflate counts for what it holds, not for what it weighs on the network.
- A `md` that a node reads without utilities must be text (UTF-8).

A download that fails stops the run before the first node, with `io` on the `url`: a host that cannot be reached, an
answer that is not 2xx, a scheme other than `http` or `https`, the 60 seconds or the 64 MiB passed, a `md` that is
not text. The message names the media or the prompt and never holds the url, whose query may hold a secret.

## Producing media

An entry of `media.output` says how the media is made:

- `from: model` (the default): the model makes it, so the media must have `generate` (`media_not_generated`). The
  entry may have a `utility`, which changes the file the model made and must end with a file.
- `from: message`: a utility makes it from the text of the node, after the utilities of `message.output`. The media
  must be of type `file` (`media_not_made`) and the entry must have a `utility` (`media_needs_utility`), which must end
  with a file.

Each entry decides its own destinations: `assistant: true` puts it in the final output, `node: true` passes it to the
node connected after. Both are `false` by default.

A media is written by one node only: two nodes that produce the same media make the file invalid
(`media_two_writers`). The node only cites the id: what the media is made of is written in `resources.media`.

## `generate` and `provider`

Every value of `generate` is given to the provider as it is written. A provider that does not accept a value makes the
file invalid when it is checked (`not_supported`).

- `voice`: mandatory for an audio on OpenAI.
- `size`: `width` and `height` together; the provider may accept only some pairs.
- `quality`: the values depend on the model.
- `background`: `true` transparent, `false` opaque; without it the provider decides. A transparent background needs an
  image of type png or webp (`media_type_unsupported`).
- `language_code`: no provider accepts it at present.

`provider` holds the fields of one family only (`exactly_one`), and it must be the family of the model of the node
that writes the media:

- `openai` (also `azure_foundry` with publisher `openai`): `moderation` (`low`, `auto`), `output_compression` (0 to
  100, for an image of type jpeg or webp), `style` (`vivid`, `natural`; not together with the other two,
  `media_provider_conflict`). No model accepts `style` at present: the GPT image models take the style from the
  prompt, and a node that asks for it makes the file invalid when it is checked (`not_supported`);
- `gemini` (also `vertex_ai` with publisher `google`): `image_size` (`512`, `1K`, `2K`, `4K`) and `aspect_ratio`.

One media is one image or one audio: a model produces one media per call.

What each provider can produce is listed in [models](models.md#what-each-provider-can-do).
