//! A produced speech, from `POST /v1/audio/speech`.

use serde_json::{Value, json};

use super::{Mistral, PROVIDER};
use crate::resources::MediaKind;
use crate::models::timing::timed;
use crate::models::{MediaOutput, ModelError, ModelInvocation, ModelResponse, ProducedMedia, ResponseInfo, StopReason, Usage};

fn audio_format(format: &str) -> String {
    match format.to_lowercase().as_str() {
        // 16-bit PCM is called pcm16 in the yml, `pcm` by Mistral
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

impl Mistral {
    /// `Ok(None)`: no media is produced. `Ok(Some(format))`: a speech in that format.
    pub(super) fn speech_format(&self, produced: &[ProducedMedia]) -> Result<Option<String>, Vec<String>> {
        let [media] = produced else {
            return if produced.is_empty() { Ok(None) } else { Err(vec!["Mistral produces one media per call".to_string()]) };
        };
        let MediaKind::Audio(format) = &media.kind else {
            return Err(vec!["Mistral produces audio only (speech)".to_string()]);
        };
        let g = &media.generate;
        let mut problems = Vec::new();
        for (name, present) in [
            ("size", g.size.is_some()),
            ("quality", g.quality.is_some()),
            ("background", g.background.is_some()),
            ("language_code", g.language_code.is_some()),
            ("provider", media.provider.is_some()),
        ] {
            if present {
                problems.push(format!("`{name}` is not accepted by Mistral speech"));
            }
        }
        if problems.is_empty() { Ok(Some(audio_format(format))) } else { Err(problems) }
    }

    pub(super) async fn invoke_speech(&self, call: &ModelInvocation, format: &str) -> Result<ModelResponse, ModelError> {
        let mut body = json!({ "model": self.model, "input": call.user, "response_format": format });
        if let Some(voice) = &call.output_media[0].generate.voice {
            body["voice_id"] = json!(voice);
        }
        let (text, secs) = timed(self.post_text("/v1/audio/speech", &body)).await;
        let v: Value = serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        let data = v.get("audio_data").and_then(Value::as_str).ok_or_else(|| self.who.invalid("no `audio_data`"))?;
        Ok(ModelResponse {
            message: None,
            structured: None,
            media: vec![MediaOutput { mime_type: audio_mime(format), data: data.to_string(), id: None }],
            tools_call: Vec::new(),
            reasoning: Vec::new(),
            stop_reason: StopReason::End,
            stop_detail: None,
            info: ResponseInfo { provider: PROVIDER.into(), model: self.model.clone(), execution_time: secs, usage: Usage::default() },
        })
    }
}
