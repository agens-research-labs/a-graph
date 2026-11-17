//! Produced media: an image from `/v1/images/generations`, a speech from `/v1/audio/speech`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value, json};

use super::OpenAi;
use crate::models::timing::timed;
use crate::models::{
    MediaOutput, ModelError, ModelInvocation, ModelResponse, ProducedMedia, ResponseInfo, StopReason, Usage,
};
use crate::resources::{MediaKind, MediaProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Image,
    Speech,
}

fn image_format(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "jpg" => "jpeg".into(),
        other => other.into(),
    }
}

fn speech_format(format: &str) -> String {
    match format.to_lowercase().as_str() {
        // 16-bit PCM is called pcm16 in the yml, `pcm` by OpenAI
        "pcm16" | "l16" => "pcm".into(),
        other => other.into(),
    }
}

fn audio_mime(format: &str) -> String {
    match format {
        "mp3" => "audio/mpeg".into(),
        "pcm" => "audio/l16".into(),
        other => format!("audio/{other}"),
    }
}

impl OpenAi {
    /// `Ok(None)`: no media is produced. `Err`: everything OpenAI cannot do with what is asked.
    pub(super) fn media_kind(&self, produced: &[ProducedMedia]) -> Result<Option<Kind>, Vec<String>> {
        let [media] = produced else {
            return if produced.is_empty() {
                Ok(None)
            } else {
                Err(vec!["OpenAI produces one media per call".to_string()])
            };
        };
        let mut problems = Vec::new();
        let g = &media.generate;
        if matches!(media.provider, Some(MediaProvider::Gemini(_))) {
            problems.push("the media provider fields are for gemini, not openai".to_string());
        }
        if g.language_code.is_some() {
            problems
                .push("`language_code` is not accepted by OpenAI: the language is detected from the text".to_string());
        }
        let kind = match &media.kind {
            MediaKind::Img(_) => {
                if g.voice.is_some() {
                    problems.push("`voice` is for audio".to_string());
                }
                if matches!(&media.provider, Some(MediaProvider::Openai(o)) if o.style.is_some()) {
                    problems.push(
                        "`style` is not accepted by the GPT image models: describe the style in the prompt".to_string(),
                    );
                }
                Kind::Image
            }
            MediaKind::Audio(_) => {
                if g.voice.is_none() {
                    problems.push("`voice` is mandatory for audio".to_string());
                }
                for (name, present) in [
                    ("size", g.size.is_some()),
                    ("quality", g.quality.is_some()),
                    ("background", g.background.is_some()),
                    ("provider", media.provider.is_some()),
                ] {
                    if present {
                        problems.push(format!("`{name}` is for images"));
                    }
                }
                Kind::Speech
            }
            MediaKind::Md | MediaKind::Video(_) | MediaKind::File => {
                problems.push("OpenAI produces images and audio only here (not text media or video)".to_string());
                return Err(problems);
            }
        };
        if problems.is_empty() {
            Ok(Some(kind))
        } else {
            Err(problems)
        }
    }

    pub(super) async fn invoke_media(&self, call: &ModelInvocation, kind: Kind) -> Result<ModelResponse, ModelError> {
        let media = &call.output_media[0];
        let prompt = if call.system.is_empty() {
            call.user.clone()
        } else {
            format!("{}\n\n{}", call.system, call.user)
        };
        match kind {
            Kind::Image => self.image(media, &prompt).await,
            Kind::Speech => self.speech(call, media).await,
        }
    }

    pub(super) fn image_body(&self, media: &ProducedMedia, prompt: &str) -> Value {
        let g = &media.generate;
        let mut b = Map::new();
        b.insert("model".into(), json!(self.model));
        b.insert("prompt".into(), json!(prompt));
        b.insert("n".into(), json!(1));
        if let MediaKind::Img(f) = &media.kind {
            b.insert("output_format".into(), json!(image_format(f)));
        }
        if let Some(s) = &g.size {
            b.insert("size".into(), json!(format!("{}x{}", s.width, s.height)));
        }
        if let Some(q) = &g.quality {
            b.insert("quality".into(), json!(q));
        }
        match g.background {
            Some(true) => b.insert("background".into(), json!("transparent")),
            Some(false) => b.insert("background".into(), json!("opaque")),
            None => None,
        };
        if let Some(MediaProvider::Openai(o)) = &media.provider {
            if let Some(m) = o.moderation {
                b.insert("moderation".into(), json!(format!("{m:?}").to_lowercase()));
            }
            if let Some(c) = o.output_compression {
                b.insert("output_compression".into(), json!(c));
            }
        }
        Value::Object(b)
    }

    async fn image(&self, media: &ProducedMedia, prompt: &str) -> Result<ModelResponse, ModelError> {
        let body = self.image_body(media, prompt);
        let (text, secs) = timed(self.post_text("/images/generations", &body)).await;
        let v: Value =
            serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        let item = v
            .pointer("/data/0")
            .ok_or_else(|| self.who.invalid("no image in `data`"))?;

        // gpt-image models always answer base64; an address is never downloaded: its host is the server's choice
        let data = item
            .get("b64_json")
            .and_then(Value::as_str)
            .ok_or_else(|| self.who.invalid("the image has no `b64_json`"))?
            .to_string();
        let produced = v.get("output_format").and_then(Value::as_str).map(str::to_string);
        let asked = match &media.kind {
            MediaKind::Img(f) => image_format(f),
            _ => unreachable!("a speech is not an image"),
        };
        let format = produced.unwrap_or_else(|| asked.clone());
        if format != asked {
            return Err(self
                .who
                .mime_mismatch(&format!("image/{asked}"), &format!("image/{format}")));
        }

        let n = |k: &str| v.pointer(&format!("/usage/{k}")).and_then(Value::as_u64).unwrap_or(0);
        let usage = Usage {
            input_tokens: n("input_tokens"),
            output_tokens: n("output_tokens"),
            ..Usage::default()
        };
        Ok(self.media_response(
            MediaOutput {
                mime_type: format!("image/{format}"),
                data,
                id: None,
            },
            usage,
            secs,
        ))
    }

    async fn speech(&self, call: &ModelInvocation, media: &ProducedMedia) -> Result<ModelResponse, ModelError> {
        let MediaKind::Audio(f) = &media.kind else {
            unreachable!("an image is not a speech")
        };
        let format = speech_format(f);
        let mut body = json!({ "model": self.model, "input": call.user, "voice": media.generate.voice, "response_format": format });
        // how to speak (gpt-4o-mini-tts)
        if !call.system.is_empty() {
            body["instructions"] = json!(call.system);
        }
        let (bytes, secs) = timed(self.post_bytes("/audio/speech", &body)).await;
        let data = STANDARD.encode(bytes?);
        Ok(self.media_response(
            MediaOutput {
                mime_type: audio_mime(&format),
                data,
                id: None,
            },
            Usage::default(),
            secs,
        ))
    }

    fn media_response(&self, media: MediaOutput, usage: Usage, execution_time: f64) -> ModelResponse {
        ModelResponse {
            message: None,
            structured: None,
            media: vec![media],
            tools_call: Vec::new(),
            reasoning: Vec::new(),
            logprobs: Vec::new(),
            stop_reason: StopReason::End,
            stop_detail: None,
            info: ResponseInfo {
                provider: self.who.provider.into(),
                model: self.model.clone(),
                execution_time,
                usage,
            },
        }
    }
}
