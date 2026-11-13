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

**A media that is read** has a `source`: exactly one of `path` (a file), `url` (with an optional `access`) and
`base64` (the content itself, with an optional `mime_type`, its type, given to a utility).

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
- a media from a `url` is not read yet: a node that reads one fails when it runs.

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
