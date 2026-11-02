//! What a yml that comes from outside may not do. The checks are made on the text, before the
//! program is built, because building already opens some files. A path is moved into the folder
//! of the session, so everything that reads or writes a file stays there.

use std::path::{Component, Path as FsPath, PathBuf};

use serde_norway::Value;

use super::ServiceConfig;
use crate::error::{Error, Path, Phase, Rule};
use crate::net::{allowed_entry, entry_text};

/// The empty file that tells a folder of `data_dir` was made by the service for a session: the sweep deletes no other.
pub(super) const SESSION_MARKER: &str = ".a-graph-session";

/// A session is a folder name: letters, digits, `-`, `_` and `.`, at most 64, and not made of dots only.
pub(super) fn valid_session(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && s.chars().any(|c| c != '.')
}

fn refuse(at: Path, rule: Rule, msg: impl Into<String>) -> Error {
    Error::new(Phase::Validation, at, rule, msg)
}

/// The yml, ready to be built: its paths inside `<data_dir>/<session>/`. `Err` has every rule it breaks.
pub fn prepare(yml: &str, config: &ServiceConfig, session: &str) -> Result<String, Vec<Error>> {
    let mut root: Value =
        serde_norway::from_str(yml).map_err(|e| vec![refuse(Path::root(), Rule::Parse, e.to_string())])?;
    let folder = config.data_dir.join(session);
    let mut errs = Vec::new();

    no_programs(&root, &mut errs);
    only_allowed_hosts(&root, config, &mut errs);
    // the places where a file is named: a list of keys leading to the path, with the list they sit in
    for (list, item_path) in [
        (&["resources", "prompts"][..], &["source", "path"][..]),
        (&["resources", "media"], &["source", "path"]),
        (&["resources", "skills"], &["path"]),
        (&["external", "memory"], &["path"]),
    ] {
        for_each_item(&mut root, list, |i, item| {
            let at = path_of(list).idx(i);
            confine(item, item_path, &at, &folder, &mut errs);
        });
    }
    for_each_item(&mut root, &["blackboards"], |i, item| {
        confine(
            item,
            &["source", "file"],
            &Path::root().key("blackboards").idx(i),
            &folder,
            &mut errs,
        );
    });
    confine(
        &mut root,
        &["observability", "output", "file"],
        &Path::root(),
        &folder,
        &mut errs,
    );

    if !errs.is_empty() {
        return Err(errs);
    }
    serde_norway::to_string(&root).map_err(|e| vec![refuse(Path::root(), Rule::Parse, e.to_string())])
}

fn path_of(keys: &[&str]) -> Path {
    keys.iter().fold(Path::root(), |p, k| p.key(k))
}

fn get<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().try_fold(v, |v, k| v.get(*k))
}

fn get_mut<'a>(v: &'a mut Value, keys: &[&str]) -> Option<&'a mut Value> {
    keys.iter().try_fold(v, |v, k| v.get_mut(*k))
}

/// Calls `f` for each item of the list at `keys`, with its position.
fn for_each_item(root: &mut Value, keys: &[&str], mut f: impl FnMut(usize, &mut Value)) {
    if let Some(Value::Sequence(items)) = get_mut(root, keys) {
        for (i, item) in items.iter_mut().enumerate() {
            f(i, item);
        }
    }
}

/// A program is never started for a graph from outside: no `stdio`, no scripts of a skill.
fn no_programs(root: &Value, errs: &mut Vec<Error>) {
    for list in [
        &["resources", "tools"][..],
        &["resources", "mcp_servers"],
        &["utilities"],
    ] {
        if let Some(Value::Sequence(items)) = get(root, list) {
            for (i, item) in items.iter().enumerate() {
                if item.get("transport").and_then(Value::as_str) == Some("stdio") {
                    errs.push(refuse(
                        path_of(list).idx(i).key("transport"),
                        Rule::NotSupported,
                        "this service does not start programs: use `transport: http`",
                    ));
                }
            }
        }
    }
    if let Some(Value::Sequence(items)) = get(root, &["resources", "skills"]) {
        for (i, item) in items.iter().enumerate() {
            if item.get("run_scripts").and_then(Value::as_bool) != Some(false) {
                errs.push(refuse(
                    path_of(&["resources", "skills"]).idx(i),
                    Rule::NotSupported,
                    "the scripts of a skill start programs: write `run_scripts: false`",
                ));
            }
        }
    }
}

/// The hosts the yml allows must all be hosts the service allows: the yml can only restrict. An
/// entry `host` or `host:port` of the yml is allowed by the same entry, or by `host` alone; `host`
/// alone is not allowed by `host:port` (it would allow every port).
fn only_allowed_hosts(root: &Value, config: &ServiceConfig, errs: &mut Vec<Error>) {
    let keys = ["passport", "adl", "permissions", "network", "allowed_hosts"];
    if let Some(Value::Sequence(hosts)) = get(root, &keys) {
        for (i, h) in hosts.iter().enumerate() {
            let Some(entry) = h.as_str() else { continue };
            match allowed_entry(entry) {
                Err(why) => errs.push(refuse(path_of(&keys).idx(i), Rule::BadValue, why)),
                Ok((host, port))
                    if !config.allowed_hosts.contains(&host)
                        && !config.allowed_hosts.contains(&entry_text(&host, port)) =>
                {
                    errs.push(refuse(
                        path_of(&keys).idx(i),
                        Rule::NetworkNotAllowed,
                        format!("host `{entry}` is not allowed by this service"),
                    ));
                }
                Ok(_) => {}
            }
        }
    }
}

/// Replaces the path at `keys` of `item` with the same path inside `folder`; an absolute path or one with `..` is an error.
fn confine(item: &mut Value, keys: &[&str], at: &Path, folder: &FsPath, errs: &mut Vec<Error>) {
    let Some(Value::String(path)) = get_mut(item, keys) else {
        return;
    };
    let here = keys.iter().fold(at.clone(), |p, k| p.key(k));
    let wanted = PathBuf::from(path.as_str());
    let escapes = wanted.is_absolute()
        || wanted
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir));
    if path.is_empty() || escapes {
        errs.push(refuse(
            here,
            Rule::NotSupported,
            format!("`{path}` is outside the folder of the session: write a relative path without `..`"),
        ));
        return;
    }
    // the mark at the top of the folder is emptied by the service at every use: a file of the graph there would be
    // lost. Past any `./`, and in any case, since a filesystem that ignores case has one file for both
    let first = wanted.components().find(|c| !matches!(c, Component::CurDir));
    if first.is_some_and(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| s.eq_ignore_ascii_case(SESSION_MARKER))
    }) {
        errs.push(refuse(
            here,
            Rule::NotSupported,
            format!("`{path}` is the mark the service keeps in the folder of the session: choose another name"),
        ));
        return;
    }
    *path = folder.join(wanted).to_string_lossy().to_string();
}
