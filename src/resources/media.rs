//! `resources.media`: media a node receives, or media a node produces.

use serde::Deserialize;
use serde_norway::Value;

use super::common::{Errs, Obj, list_path};
use super::source::{self, Source};
use crate::error::{Errors, Path, Rule};
use crate::network::Destination;
use crate::schema::key;

/// `md`, or the format of an image, an audio, a video.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Md,
    /// A file of any format, made by a utility from the text of a node: the program does not look inside.
    File,
    Img(String),
    Audio(String),
    Video(String),
}

#[derive(Debug, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// How a produced media is created; every value goes to the provider as written.
#[derive(Debug, Clone, Default)]
pub struct Generate {
    pub voice: Option<String>,
    pub language_code: Option<String>,
    pub size: Option<Size>,
    pub quality: Option<String>,
    pub background: Option<bool>,
}

/// The value of `moderation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Moderation {
    Low,
    Auto,
}

/// The value of `style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageStyle {
    Vivid,
    Natural,
}

#[derive(Debug, Clone)]
pub struct OpenaiMedia {
    pub moderation: Option<Moderation>,
    pub output_compression: Option<u32>,
    pub style: Option<ImageStyle>,
}

/// The value of `image_size`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiImageSize {
    S512,
    K1,
    K2,
    K4,
}

#[derive(Debug, Clone)]
pub struct GeminiMedia {
    pub image_size: Option<GeminiImageSize>,
    pub aspect_ratio: Option<String>,
}

/// The fields of ONE family of providers.
#[derive(Debug, Clone)]
pub enum MediaProvider {
    Openai(OpenaiMedia),
    Gemini(GeminiMedia),
}

#[derive(Debug, Clone)]
pub enum Origin {
    /// An input media, read from a source.
    Source(Source),
    /// A media made by a utility from the text of a node (`type: file`): it has neither a source nor `generate`.
    MadeByUtility,
    /// A media produced by a node.
    Generated {
        generate: Generate,
        provider: Option<MediaProvider>,
    },
}

#[derive(Debug, Clone)]
pub struct Media {
    pub id: String,
    pub kind: MediaKind,
    pub origin: Origin,
}

#[derive(Debug, Default)]
pub struct MediaManager {
    media: Vec<Media>,
}

impl MediaManager {
    /// Builds from the value of `resources.media`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut media = Vec::new();
        for mut o in Obj::items(&list_path(key::MEDIA), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            errs.require(&at, key::TYPE, o.has(key::TYPE), "`type` is mandatory");
            let kind = o
                .value(key::TYPE)
                .and_then(|v| read_kind(v, &at.key(key::TYPE), &mut errs));
            let (has_source, has_generate) = (o.has(key::SOURCE), o.has(key::GENERATE));
            let src = o.sub(key::SOURCE, &mut errs);
            let generate = o.sub(key::GENERATE, &mut errs).map(|g| read_generate(g, &mut errs));
            let provider = o.sub(key::PROVIDER, &mut errs);

            let origin = match (has_source, has_generate) {
                (false, false) if kind == Some(MediaKind::File) => Some(Origin::MadeByUtility),
                (true, _) | (_, true) if kind == Some(MediaKind::File) => {
                    errs.add(
                        &at,
                        Rule::ExactlyOne,
                        "a media of type `file` has neither `source` nor `generate`: a utility makes it",
                    );
                    None
                }
                (true, false) => {
                    errs.forbid(&at, key::PROVIDER, provider.is_some(), "provider goes with `generate`");
                    src.and_then(|s| source::build(s, true, &mut errs)).map(Origin::Source)
                }
                (false, true) => {
                    let generate = generate.unwrap_or_default();
                    let provider = read_provider(&at, kind.as_ref(), &generate, provider, &mut errs);
                    Some(Origin::Generated { generate, provider })
                }
                _ => {
                    errs.add(&at, Rule::ExactlyOne, "a media needs exactly one of source, generate");
                    None
                }
            };
            if let (Some(id), Some(kind), Some(origin), true) = (id, kind, origin, errs.0.len() == before) {
                media.push(Media { id, kind, origin });
            }
        }
        errs.finish(MediaManager { media })
    }

    /// The urls media are read from, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = list_path(key::MEDIA);
        self.media
            .iter()
            .enumerate()
            .filter_map(|(i, m)| match &m.origin {
                Origin::Source(Source::Url { url, .. }) => {
                    Some(Destination::url(base.idx(i).key(key::SOURCE).key(key::URL), url))
                }
                _ => None,
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Media> {
        self.media.iter().find(|m| m.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Media> {
        self.media.iter()
    }
}

/// `md`, or a map with one of `img`, `audio`, `video` and the format.
fn read_kind(v: Value, at: &Path, errs: &mut Errs) -> Option<MediaKind> {
    if v.as_str() == Some("md") {
        return Some(MediaKind::Md);
    }
    if v.as_str() == Some("file") {
        return Some(MediaKind::File);
    }
    let mut o = Obj::new(v, at.clone(), errs)?;
    let (img, audio, video) = (
        o.opt::<String>(key::IMG, errs),
        o.opt::<String>(key::AUDIO, errs),
        o.opt::<String>(key::VIDEO, errs),
    );
    match (img, audio, video) {
        (Some(f), None, None) => Some(MediaKind::Img(f)),
        (None, Some(f), None) => Some(MediaKind::Audio(f)),
        (None, None, Some(f)) => Some(MediaKind::Video(f)),
        _ => {
            errs.add(
                at,
                Rule::ExactlyOne,
                "type is `md`, `file` or one of img, audio, video with its format",
            );
            None
        }
    }
}

fn read_generate(mut g: Obj, errs: &mut Errs) -> Generate {
    let size = g.sub(key::SIZE, errs).and_then(|mut s| {
        let (w, h) = (s.req::<u32>(key::WIDTH, errs), s.req::<u32>(key::HEIGHT, errs));
        Some(Size { width: w?, height: h? })
    });
    Generate {
        voice: g.opt(key::VOICE, errs),
        language_code: g.opt(key::LANGUAGE_CODE, errs),
        size,
        quality: g.opt(key::QUALITY, errs),
        background: g.opt(key::BACKGROUND, errs),
    }
}

/// The rules of a produced media: the format that allows transparency and compression,
/// and the fields of the provider family.
fn read_provider(
    at: &Path,
    kind: Option<&MediaKind>,
    g: &Generate,
    raw: Option<Obj>,
    errs: &mut Errs,
) -> Option<MediaProvider> {
    let format = match kind {
        Some(MediaKind::Img(f)) => Some(f.to_lowercase()),
        _ => None,
    };
    let is = |formats: &[&str]| format.as_deref().is_some_and(|f| formats.contains(&f));

    if g.background == Some(true) && !is(&["png", "webp"]) {
        errs.add(
            &at.key(key::GENERATE).key(key::BACKGROUND),
            Rule::MediaTypeUnsupported,
            "transparency needs an img of type png or webp",
        );
    }
    let mut raw = raw?;
    let (openai, gemini) = (raw.sub(key::OPENAI, errs), raw.sub(key::GEMINI, errs));
    match (openai, gemini) {
        (Some(_), Some(_)) => {
            errs.add(
                &at.key(key::PROVIDER),
                Rule::ExactlyOne,
                "provider holds the fields of ONE family only",
            );
            None
        }
        (Some(mut o), None) => {
            let op = at.key(key::PROVIDER).key(key::OPENAI);
            let m = OpenaiMedia {
                moderation: o.opt(key::MODERATION, errs),
                output_compression: o.opt(key::OUTPUT_COMPRESSION, errs),
                style: o.opt(key::STYLE, errs),
            };
            if let Some(c) = m.output_compression {
                if c > 100 {
                    errs.add(
                        &op.key(key::OUTPUT_COMPRESSION),
                        Rule::BadValue,
                        "output_compression is 0-100",
                    );
                }
                if !is(&["jpeg", "jpg", "webp"]) {
                    errs.add(
                        &op.key(key::OUTPUT_COMPRESSION),
                        Rule::MediaTypeUnsupported,
                        "output_compression needs an img of type jpeg or webp",
                    );
                }
            }
            if m.style.is_some() && (m.moderation.is_some() || m.output_compression.is_some()) {
                errs.add(
                    &op.key(key::STYLE),
                    Rule::MediaProviderConflict,
                    "style cannot be used with moderation or output_compression",
                );
            }
            Some(MediaProvider::Openai(m))
        }
        (None, Some(mut g)) => {
            let image_size = g.opt_text(key::IMAGE_SIZE, errs).and_then(|t| match t.as_str() {
                "512" => Some(GeminiImageSize::S512),
                "1K" => Some(GeminiImageSize::K1),
                "2K" => Some(GeminiImageSize::K2),
                "4K" => Some(GeminiImageSize::K4),
                other => {
                    errs.add(
                        &g.at().key(key::IMAGE_SIZE),
                        Rule::BadValue,
                        format!("`{other}` is not one of 512, 1K, 2K, 4K"),
                    );
                    None
                }
            });
            Some(MediaProvider::Gemini(GeminiMedia {
                image_size,
                aspect_ratio: g.opt(key::ASPECT_RATIO, errs),
            }))
        }
        (None, None) => None,
    }
}
