//! Where an error is: a path in the yml, or a node id.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Key(String),
    Index(usize),
    /// A node known only by its id (execution errors have no yml position).
    Node(String),
}

/// Where the error is: a path in the yml, or a node id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Path(pub Vec<Segment>);

impl Path {
    pub fn root() -> Self {
        Path(Vec::new())
    }
    pub fn node(id: &str) -> Self {
        Path(vec![Segment::Node(id.to_string())])
    }
    pub fn key(&self, k: &str) -> Self {
        let mut p = self.clone();
        p.0.push(Segment::Key(k.to_string()));
        p
    }
    pub fn idx(&self, i: usize) -> Self {
        let mut p = self.clone();
        p.0.push(Segment::Index(i));
        p
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("<root>");
        }
        for (n, s) in self.0.iter().enumerate() {
            match s {
                Segment::Key(k) if n == 0 => write!(f, "{k}")?,
                Segment::Key(k) => write!(f, ".{k}")?,
                Segment::Index(i) => write!(f, "[{i}]")?,
                Segment::Node(id) => write!(f, "node({id})")?,
            }
        }
        Ok(())
    }
}
