//! The error type: plain data. It never logs itself; the logging sink consumes it as an event.

use std::fmt;

mod path;
mod rule;

pub use path::{Path, Segment};
pub use rule::Rule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Validation,
    Interpretation,
    Execution,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Phase::Validation => "validation",
            Phase::Interpretation => "interpretation",
            Phase::Execution => "execution",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub phase: Phase,
    pub path: Path,
    pub rule: Rule,
    pub message: String,
}

impl Error {
    pub fn new(phase: Phase, path: Path, rule: Rule, message: impl Into<String>) -> Self {
        Error {
            phase,
            path,
            rule,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}): {}",
            self.phase,
            self.path,
            self.rule.code(),
            self.message
        )
    }
}

impl std::error::Error for Error {}

/// Something worth saying that does not stop the build. Plain data, like `Error`: it never
/// logs itself, whoever coordinates reads it and sends it to the logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub phase: Phase,
    pub path: Path,
    /// Stable name of what happened.
    pub code: &'static str,
    pub message: String,
}

impl Warning {
    pub fn new(phase: Phase, path: Path, code: &'static str, message: impl Into<String>) -> Self {
        Warning {
            phase,
            path,
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} ({}): {}", self.phase, self.path, self.code, self.message)
    }
}

/// A list of errors: validation reports all of them in one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errors(pub Vec<Error>);

impl Errors {
    pub fn rules(&self) -> Vec<&'static str> {
        self.0.iter().map(|e| e.rule.code()).collect()
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (n, e) in self.0.iter().enumerate() {
            if n > 0 {
                writeln!(f)?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Errors {}
