use crate::common::logging::*;

/// Every write the output receives, as it came.
#[derive(Clone, Default)]
struct Writes(Arc<Mutex<Vec<Vec<u8>>>>);

impl Write for Writes {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().push(b.to_vec());
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A line is written whole, newline included, in one write: the runs of a session append to the same file, each
/// with its own logger, and two writes per line could interleave.
#[test]
fn one_write_per_line() {
    let writes = Writes::default();
    let log = Logger::new(LogLevel::Debug, Redaction { allow_content: true }, Box::new(writes.clone()));
    log.log(&Event::new(LogLevel::Info, "a"));
    log.log(&Event::new(LogLevel::Info, "b"));
    let writes = writes.0.lock().unwrap();
    assert_eq!(writes.len(), 2, "{writes:?}");
    for w in writes.iter() {
        assert_eq!(w.iter().filter(|b| **b == b'\n').count(), 1);
        assert_eq!(w.last(), Some(&b'\n'));
    }
}
