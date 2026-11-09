//! The verbose reporter: the human-facing progress of a run, written as it happens. It reads the
//! same events as the logger. `steps` shows the nodes and the groups, `detail` adds the calls to
//! the models, the tools and the readings. Colors appear only on a terminal.

use std::io::{self, IsTerminal, Write};
use std::sync::Mutex;

use serde::Deserialize;
use serde_json::Value;
use serde_norway::Value as Yml;

use super::{Event, LogLevel, Redaction};
use crate::build::{Errs, Obj};
use crate::error::{Errors, Path};
use crate::schema::key;

/// The value of `verbose.level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerboseLevel {
    Off,
    Steps,
    Detail,
}

/// How many characters of a text are shown on one line.
const SHOWN: usize = 200;

pub struct Verbose {
    level: VerboseLevel,
    redaction: Redaction,
    color: bool,
    out: Mutex<Box<dyn Write + Send>>,
}

impl Verbose {
    pub fn new(level: VerboseLevel, redaction: Redaction, color: bool, out: Box<dyn Write + Send>) -> Self {
        Verbose {
            level,
            redaction,
            color,
            out: Mutex::new(out),
        }
    }

    /// To the standard error, in color when it is a terminal and `NO_COLOR` is not set.
    pub fn to_stderr(level: VerboseLevel, redaction: Redaction) -> Self {
        let color = io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        Self::new(level, redaction, color, Box::new(io::stderr()))
    }

    /// Builds from the value of the `verbose` block.
    pub fn build(block: Option<Yml>, redaction: Redaction) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let level = block
            .and_then(|b| Obj::new(b, Path::root().key(key::VERBOSE), &mut errs))
            .and_then(|mut o| o.req::<VerboseLevel>(key::LEVEL, &mut errs));
        errs.finish(Verbose::to_stderr(level.unwrap_or(VerboseLevel::Off), redaction))
    }

    pub fn level(&self) -> VerboseLevel {
        self.level
    }

    /// Whether message contents may be shown (the passport allows them).
    pub fn allow_content(&self) -> bool {
        self.redaction.allow_content
    }

    /// Writes what the event shows at this level, if anything.
    pub fn show(&self, ev: &Event) {
        if self.level == VerboseLevel::Off {
            return;
        }
        let Some(text) = self.render(ev) else { return };
        let mut out = self.out.lock().unwrap();
        // a failing display must not stop the run
        let _ = writeln!(out, "{text}").and_then(|_| out.flush());
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn render(&self, ev: &Event) -> Option<String> {
        let detail = self.level == VerboseLevel::Detail;
        let f = |k: &str| ev.fields.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        let text = |k: &str| f(k).and_then(Value::as_str).unwrap_or_default().to_string();
        let number = |k: &str| f(k).and_then(Value::as_f64).unwrap_or_default();
        let seconds = |k: &str| format!("{:.1} s", number(k));
        let node = ev.node.clone().unwrap_or_default();
        let bar = "│ ".repeat(ev.depth);
        let inner = format!("{bar}  ");

        match ev.code.as_str() {
            "run.start" => Some(self.paint("1", "run started")),
            "run.end" => Some(self.paint("1", &format!("run finished · {}", seconds("duration")))),
            "node.start" => Some(format!(
                "{bar}{}{}",
                self.paint("1;36", &format!("▶ {node}")),
                self.paint("90", &format!(" · {}", text("model")))
            )),
            "node.end" => {
                let verdict = f("verdict")
                    .and_then(Value::as_str)
                    .map(|v| format!(" · {v}"))
                    .unwrap_or_default();
                let tokens = format!(
                    " · {} in / {} out tokens",
                    number("input_tokens"),
                    number("output_tokens")
                );
                Some(format!(
                    "{bar}{}{}",
                    self.paint("1;36", &format!("✓ {node}")),
                    self.paint("90", &format!(" · {}{tokens}{verdict}", seconds("duration")))
                ))
            }
            "group.start" => Some(format!(
                "{bar}{}",
                self.paint(
                    "1;38;5;208",
                    &format!("┌─ {} · {} branches", text("pattern"), number("branches"))
                )
            )),
            "group.end" => Some(format!(
                "{bar}{}",
                self.paint(
                    "1;38;5;208",
                    &format!("└─ {} done · {}", text("pattern"), seconds("duration"))
                )
            )),
            "model.call" if detail => {
                let mut lines = vec![format!(
                    "{inner}{}",
                    self.paint(
                        "90",
                        &format!(
                            "model {} · {} in / {} out tokens · {}",
                            text("model"),
                            number("input_tokens"),
                            number("output_tokens"),
                            seconds("duration")
                        )
                    )
                )];
                lines.extend(self.contents(ev, &inner));
                Some(lines.join("\n"))
            }
            "tool.call" if detail => {
                let failed = f("is_error").and_then(Value::as_bool).unwrap_or(false);
                let head = format!(
                    "⟶ {} {} · {}{}",
                    text("kind"),
                    text("name"),
                    seconds("duration"),
                    if failed { " · error" } else { "" }
                );
                let mut lines = vec![format!(
                    "{inner}{}",
                    self.paint(if failed { "31" } else { "34" }, &head)
                )];
                lines.extend(self.contents(ev, &inner));
                Some(lines.join("\n"))
            }
            "utility.call" if detail => Some(format!(
                "{inner}{}",
                self.paint(
                    "90",
                    &format!("utility {} · {} · {}", text("id"), text("place"), seconds("duration"))
                )
            )),
            "read.vectordb" if detail => Some(format!(
                "{inner}{}",
                self.paint(
                    "90",
                    &format!("read vectordb {} · {} chunks", text("id"), number("chunks"))
                )
            )),
            "read.memory" if detail => Some(format!(
                "{inner}{}",
                self.paint(
                    "90",
                    &format!(
                        "read memory {} · {} messages{}",
                        text("id"),
                        number("messages"),
                        if f("summary").and_then(Value::as_bool).unwrap_or(false) {
                            " · with summary"
                        } else {
                            ""
                        }
                    )
                )
            )),
            _ if ev.level == LogLevel::Error => {
                let mut lines = vec![format!(
                    "{bar}{}",
                    self.paint("31", &format!("✗ {}: {}", ev.code, text("message")))
                )];
                lines.extend(self.contents(ev, &inner));
                Some(lines.join("\n"))
            }
            _ if ev.level == LogLevel::Warn => Some(format!(
                "{bar}{}",
                self.paint("33", &format!("! {}: {}", ev.code, text("message")))
            )),
            _ => None,
        }
    }

    /// The message contents of an event, one line each; the reasoning in gray. Hidden, and said so,
    /// when the passport does not allow message contents.
    fn contents(&self, ev: &Event, inner: &str) -> Vec<String> {
        if ev.content.is_empty() {
            return Vec::new();
        }
        if !self.redaction.allow_content {
            return vec![format!(
                "{inner}{}",
                self.paint(
                    "90",
                    "(contents hidden: the data classification of the passport does not allow them)"
                )
            )];
        }
        ev.content
            .iter()
            .map(|(k, v)| {
                let flat: String = v.split_whitespace().collect::<Vec<_>>().join(" ");
                let short = if flat.chars().count() > SHOWN {
                    format!("{}…", flat.chars().take(SHOWN).collect::<String>())
                } else {
                    flat
                };
                let line = format!("{k}: {short}");
                format!(
                    "{inner}{}",
                    if k == "reasoning" || k == "result" {
                        self.paint("90", &line)
                    } else {
                        line
                    }
                )
            })
            .collect()
    }
}
