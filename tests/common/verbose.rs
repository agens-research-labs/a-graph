//! The display of a run (`verbose`) captured in memory, so a test can read what was shown.

pub use a_graph::logging::{Verbose, VerboseLevel};
pub use a_graph::logging::Redaction;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// A place the display writes to, and the test reads from.
#[derive(Clone, Default)]
pub struct Screen(pub Arc<Mutex<Vec<u8>>>);

impl Write for Screen {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Screen {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).to_string()
    }

    /// The lines shown.
    pub fn lines(&self) -> Vec<String> {
        self.text().lines().map(str::to_string).collect()
    }
}

/// Puts a display of this level in the program; the program must not be shared yet.
pub fn show(p: &mut Arc<a_graph::run::Program>, level: VerboseLevel, allow_content: bool, color: bool) -> Screen {
    let screen = Screen::default();
    Arc::get_mut(p).expect("the program is not shared yet").verbose = Verbose::new(level, Redaction { allow_content }, color, Box::new(screen.clone()));
    screen
}
