//! Small file helpers shared by the blocks.

use std::io::{self, Write};
use std::path::PathBuf;

/// Replaces the content of `path` in one step: a temporary file of its own with a random name, next to the
/// file, then renamed over it, so a reader sees the file before or after the write, never half of it. The
/// new file takes the permissions of the file it replaces; its owner is the process, and a hard link to the
/// old file keeps the old content. A symbolic link is refused and nothing is written: following it would
/// write outside the folder the path was checked against (the session folder of the service).
pub(crate) fn replace(path: &str, content: &[u8]) -> io::Result<()> {
    let permissions = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => {
            return Err(io::Error::other(
                "it is a symbolic link: point the yml at the file itself",
            ));
        }
        Ok(m) => Some(m.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let temporary = PathBuf::from(format!("{path}.{}.tmp", uuid::Uuid::new_v4().simple()));
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .and_then(|mut f| {
            // The permissions before the content, so the content is never readable by more than before.
            if let Some(p) = permissions {
                f.set_permissions(p)?;
            }
            f.write_all(content)
        })
        // A link made at `path` after the check is replaced by the rename, not followed.
        .and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        // The error of the write is the one that matters; the file may not even exist.
        let _ = std::fs::remove_file(&temporary);
    }
    written
}
