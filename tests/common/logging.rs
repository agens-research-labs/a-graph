//! The logging: every test is a folder with its own yml files.




pub use std::io::{self, Write};
pub use std::sync::{Arc, Mutex};
pub use std::time::Duration;
pub use a_graph::logging::{Delivery, Event, LogLevel, Logger, Redaction};
pub use a_graph::loader::load;
pub use a_graph::schema::key;
pub use a_graph::{Error, Path, Phase, Rule, Warning};


#[derive(Clone, Default)]
pub struct Buf(pub Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Buf {
    pub fn lines(&self) -> Vec<serde_json::Value> {
        let text = String::from_utf8(self.0.lock().unwrap().clone()).unwrap();
        text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
    }
}

pub fn logger(level: LogLevel, allow_content: bool) -> (Logger, Buf) {
    let buf = Buf::default();
    (Logger::new(level, Redaction { allow_content }, Box::new(buf.clone())), buf)
}

/// The `passport` and `observability` blocks of a yml, as the Loader hands them over.
pub fn blocks(src: &str) -> (Option<serde_norway::Value>, Option<serde_norway::Value>) {
    let mut doc = load(src).unwrap_or_else(|e| panic!("{e}"));
    (doc.take(key::PASSPORT), doc.take(key::OBSERVABILITY))
}
