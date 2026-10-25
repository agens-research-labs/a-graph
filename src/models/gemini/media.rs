//! Produced media: from the media resources of the node to `response_format` and `speech_config`.

use serde_json::{Map, Value, json};

use super::Gemini;
use crate::models::ProducedMedia;
use crate::resources::{GeminiImageSize, MediaKind, MediaProvider};

/// How one produced media is asked: what goes in `response_format` and `generation_config`.
pub(super) struct MediaFormat {
    pub(super) response_format: Value,
    pub(super) speech_config: Option<Value>,
    /// The type the answer must have.
    pub(super) mime_type: String,
}

pub(super) fn image_mime(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg".into(),
        other => format!("image/{other}"),
    }
}

fn audio_mime(format: &str) -> String {
    match format.to_lowercase().as_str() {
        // 16-bit PCM has no standard name: the format is called pcm16 in the yml
        "pcm16" | "l16" => "audio/l16".into(),
        other => format!("audio/{other}"),
    }
}

impl Gemini {
    /// `Ok(None)`: no media is produced. `Err`: everything Gemini cannot do with what is asked.
    pub(super) fn media_format(&self, produced: &[ProducedMedia]) -> Result<Option<MediaFormat>, Vec<String>> {
        let [media] = produced else {
            return if produced.is_empty() { Ok(None) } else { Err(vec!["Gemini produces one media per call".to_string()]) };
        };
        let mut problems = Vec::new();
        let g = &media.generate;
        if matches!(media.provider, Some(MediaProvider::Openai(_))) {
            problems.push("the media provider fields are for openai, not gemini".to_string());
        }
        if g.size.is_some() {
            problems.push("`size` is not accepted by Gemini: use provider.gemini image_size and aspect_ratio".to_string());
        }
        if g.quality.is_some() {
            problems.push("`quality` is not accepted by Gemini".to_string());
        }
        if g.background.is_some() {
            problems.push("`background` is not accepted by Gemini".to_string());
        }
        if g.language_code.is_some() {
            problems.push("`language_code` is not accepted by Gemini: the language is detected from the text".to_string());
        }

        let result = match &media.kind {
            MediaKind::Img(format) => {
                if g.voice.is_some() {
                    problems.push("`voice` is for audio".to_string());
                }
                let mime_type = image_mime(format);
                let mut rf = Map::new();
                rf.insert("type".into(), json!("image"));
                rf.insert("mime_type".into(), json!(mime_type));
                if let Some(MediaProvider::Gemini(gem)) = &media.provider {
                    if let Some(size) = gem.image_size {
                        let s = match size {
                            GeminiImageSize::S512 => "512",
                            GeminiImageSize::K1 => "1K",
                            GeminiImageSize::K2 => "2K",
                            GeminiImageSize::K4 => "4K",
                        };
                        rf.insert("image_size".into(), json!(s));
                    }
                    if let Some(ratio) = &gem.aspect_ratio {
                        rf.insert("aspect_ratio".into(), json!(ratio));
                    }
                }
                MediaFormat { response_format: Value::Object(rf), speech_config: None, mime_type }
            }
            MediaKind::Audio(format) => {
                if matches!(media.provider, Some(MediaProvider::Gemini(_))) {
                    problems.push("the media provider fields of gemini are for images".to_string());
                }
                let mime_type = audio_mime(format);
                MediaFormat {
                    response_format: json!({ "type": "audio", "mime_type": mime_type }),
                    speech_config: g.voice.as_ref().map(|v| json!([{ "voice": v }])),
                    mime_type,
                }
            }
            MediaKind::Md | MediaKind::Video(_) | MediaKind::File => {
                problems.push("Gemini produces images and audio only here (not text media or video)".to_string());
                return Err(problems);
            }
        };
        if problems.is_empty() { Ok(Some(result)) } else { Err(problems) }
    }
}
