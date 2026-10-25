//! JSON values and the SDK's `Document`, and the image formats.

use aws_sdk_bedrockruntime::types::ImageFormat;
use aws_smithy_types::{Document, Number};
use serde_json::Value;

pub(super) fn to_document(v: &Value) -> Document {
    match v {
        Value::Null => Document::Null,
        Value::Bool(b) => Document::Bool(*b),
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Document::Number(Number::PosInt(u))
            } else if let Some(i) = n.as_i64() {
                Document::Number(Number::NegInt(i))
            } else {
                Document::Number(Number::Float(n.as_f64().unwrap_or_default()))
            }
        }
        Value::String(s) => Document::String(s.clone()),
        Value::Array(a) => Document::Array(a.iter().map(to_document).collect()),
        Value::Object(o) => Document::Object(o.iter().map(|(k, v)| (k.clone(), to_document(v))).collect()),
    }
}

pub(super) fn from_document(d: &Document) -> Value {
    match d {
        Document::Null => Value::Null,
        Document::Bool(b) => Value::Bool(*b),
        Document::Number(Number::PosInt(u)) => Value::from(*u),
        Document::Number(Number::NegInt(i)) => Value::from(*i),
        Document::Number(Number::Float(f)) => serde_json::Number::from_f64(*f).map(Value::Number).unwrap_or(Value::Null),
        Document::String(s) => Value::String(s.clone()),
        Document::Array(a) => Value::Array(a.iter().map(from_document).collect()),
        Document::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), from_document(v))).collect()),
    }
}

pub(super) fn image_format(format: &str) -> Option<ImageFormat> {
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::Webp),
        _ => None,
    }
}
