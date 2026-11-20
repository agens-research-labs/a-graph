//! The graph: the nodes, as entities with components, and the rules that need the managers.
//! The edges are not built yet.

pub mod edges;
mod interpret;
pub mod nodes;

pub(crate) use interpret::scan;
pub use interpret::{CHUNKS, downloaded_placeholders, interpret};

/// The name of the tool of the program that gives the model the instructions of a skill.
pub const ACTIVATE_SKILL: &str = "activate_skill";

/// The name of the tool of the program that reads a file of the folder of a skill.
pub const READ_SKILL_FILE: &str = "read_skill_file";

/// The name of the tool of the program that runs a script of a skill.
pub const RUN_SKILL_SCRIPT: &str = "run_skill_script";
pub use edges::Edges;
pub use nodes::{Entity, Nodes};
