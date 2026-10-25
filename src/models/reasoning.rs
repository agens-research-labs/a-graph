//! Two small closed lists the models module owns.

use serde::Deserialize;

/// How much a model reasons: the value of `reasoning` of a node. A level the provider lacks is
/// replaced by the nearest one (`minimal` by `low`, `max` by `high`); `off` has no substitute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reasoning {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    Max,
}

/// What a model takes in (`input`) or gives back (`output`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
}
