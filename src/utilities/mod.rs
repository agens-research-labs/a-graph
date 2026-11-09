//! The utilities: programs and services, in any language, that the program calls on the data of a
//! node (never the model: that is what separates them from the tools). A utility receives one
//! JSON object and answers one JSON object. They are declared once in `utilities` and a node
//! names them by id, in order, where it wants its data changed.

mod call;
mod manager;

pub use call::{UtilityFailure, UtilityInput, UtilityOutput, call_utility, mime_of};
pub use manager::{Utility, UtilityManager, UtilityTransport};
