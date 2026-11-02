//! `resources.skills`: folders with a SKILL.md, as in the Agent Skills standard
//! (https://agentskills.io/specification).

use std::collections::BTreeMap;
use std::path::Path as FsPath;

use serde::Deserialize;
use serde_norway::Value;

use super::common::{Errs, Obj, list_path};
use crate::error::{Errors, Path, Phase, Rule, Warning};
use crate::schema::key;

/// The frontmatter of SKILL.md: the keys are the ones of the Agent Skills standard, not of our yml.
/// Fields the standard does not define are ignored.
#[derive(Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<BTreeMap<String, String>>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<String>,
    /// Every other field: not defined by the standard, ignored with a warning.
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// What is loaded at startup for every skill; the instructions are read when the skill is used.
#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    /// The folder of the skill, as written in the yml.
    pub path: String,
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: BTreeMap<String, String>,
    /// `allowed-tools` of the standard: tools pre-approved to run, so that no confirmation of the user
    /// is asked. a-graph never asks for one, so it is kept and nothing more.
    pub allowed_tools: Option<String>,
    /// The most files listed to the model; the listing says when there are more.
    pub max_files: usize,
    /// The most bytes of one file of the folder that are read for the model.
    pub max_file_bytes: u64,
    /// The deepest level of folders looked into when listing the files.
    pub max_depth: usize,
    /// Whether the model may run the scripts of the folder `scripts/`.
    pub run_scripts: bool,
    /// Seconds one run of a script may take.
    pub timeout: u32,
    /// The most calls of `read_skill_file` and `run_skill_script` for this skill in one run of a node.
    pub max_iter: u32,
}

impl Skill {
    /// The Markdown after the frontmatter, read from SKILL.md when the skill is activated.
    pub fn instructions(&self) -> std::io::Result<String> {
        let text = std::fs::read_to_string(FsPath::new(&self.path).join(FILE))?;
        let (_, body) = split(&text).ok_or_else(|| std::io::Error::other("SKILL.md has no frontmatter"))?;
        Ok(body.to_string())
    }
}

const FILE: &str = "SKILL.md";

/// The values of `max_files`, `max_file_bytes` and `max_depth` when the yml does not write them.
pub const DEFAULT_MAX_FILES: usize = 200;
pub const DEFAULT_MAX_FILE_BYTES: u64 = 100_000;
pub const DEFAULT_MAX_DEPTH: usize = 6;
/// The default of `timeout` for the scripts of a skill, in seconds.
pub const DEFAULT_TIMEOUT: u32 = 30;
/// The default of `max_iter` for a skill.
pub const DEFAULT_MAX_ITER: u32 = 20;
/// The folder of a skill whose files the model may run.
pub const SCRIPTS_DIR: &str = "scripts";

impl Skill {
    /// The files of the skill folder, other than SKILL.md, as paths relative to it written with `/`,
    /// in order. At most `max_files`; the flag says the listing is incomplete.
    pub fn files(&self) -> (Vec<String>, bool) {
        let root = FsPath::new(&self.path);
        let mut found = Vec::new();
        walk(root, root, 0, self.max_depth, &mut found);
        found.sort();
        let truncated = found.len() > self.max_files;
        found.truncate(self.max_files);
        (found, truncated)
    }

    /// The path of a script to run: a file inside the folder `scripts/` of the skill. A path
    /// outside it, with a `..`, or through a link that leads out is refused; the error says why.
    pub fn script_path(&self, relative: &str) -> Result<std::path::PathBuf, String> {
        let rel = FsPath::new(relative);
        let mut parts = rel.components();
        let inside = !rel.is_absolute()
            && parts.next().is_some_and(|c| c.as_os_str() == SCRIPTS_DIR)
            && rel.components().count() > 1
            && rel.components().all(|c| matches!(c, std::path::Component::Normal(_)));
        if !inside {
            return Err(format!(
                "`{relative}` is not a file inside the `{SCRIPTS_DIR}` folder of the skill"
            ));
        }
        let root = std::fs::canonicalize(FsPath::new(&self.path).join(SCRIPTS_DIR))
            .map_err(|_| format!("the skill has no `{SCRIPTS_DIR}` folder"))?;
        let file = std::fs::canonicalize(FsPath::new(&self.path).join(rel))
            .map_err(|_| format!("the skill has no file `{relative}`"))?;
        if !file.starts_with(&root) || !file.is_file() {
            return Err(format!(
                "`{relative}` is not a file inside the `{SCRIPTS_DIR}` folder of the skill"
            ));
        }
        Ok(file)
    }

    /// The text of a file of the skill folder. The path is relative to the folder and stays inside
    /// it: an absolute path, a `..` or a link that leads out is refused. The error says why, for the
    /// model to read.
    pub fn read_file(&self, relative: &str) -> Result<String, String> {
        let rel = FsPath::new(relative);
        if relative.is_empty()
            || rel.is_absolute()
            || rel.components().any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(format!("`{relative}` is not a path inside the skill folder"));
        }
        let root = std::fs::canonicalize(&self.path).map_err(|e| format!("cannot open the skill folder: {e}"))?;
        let file = std::fs::canonicalize(root.join(rel)).map_err(|_| format!("the skill has no file `{relative}`"))?;
        if !file.starts_with(&root) {
            return Err(format!("`{relative}` leads outside the skill folder"));
        }
        let meta = std::fs::metadata(&file).map_err(|e| format!("cannot read `{relative}`: {e}"))?;
        if !meta.is_file() {
            return Err(format!("`{relative}` is not a file"));
        }
        if meta.len() > self.max_file_bytes {
            return Err(format!(
                "`{relative}` has {} bytes, more than the {} that can be read",
                meta.len(),
                self.max_file_bytes
            ));
        }
        let bytes = std::fs::read(&file).map_err(|e| format!("cannot read `{relative}`: {e}"))?;
        String::from_utf8(bytes).map_err(|_| format!("`{relative}` is not text"))
    }
}

/// Collects the files under `dir`, relative to `root`; hidden entries and links that lead out of
/// the root are left out.
fn walk(root: &FsPath, dir: &FsPath, depth: usize, max_depth: usize, found: &mut Vec<String>) {
    if depth > max_depth {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let canonical_root = std::fs::canonicalize(root).ok();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || (depth == 0 && name == FILE) {
            continue;
        }
        // a link is followed only if it stays inside the folder
        let Ok(real) = std::fs::canonicalize(&path) else {
            continue;
        };
        if !canonical_root.as_ref().is_some_and(|r| real.starts_with(r)) {
            continue;
        }
        if real.is_dir() {
            walk(root, &path, depth + 1, max_depth, found);
        } else if let Ok(rel) = path.strip_prefix(root) {
            found.push(
                rel.components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
        }
    }
}

#[derive(Debug, Default)]
pub struct SkillsManager {
    skills: Vec<Skill>,
    warnings: Vec<Warning>,
}

impl SkillsManager {
    /// Builds from the value of `resources.skills`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut skills = Vec::new();
        let mut warnings = Vec::new();
        for mut o in Obj::items(&list_path(key::SKILLS), list, &mut errs) {
            let at = o.at().key(key::PATH);
            let (id, path) = (
                o.req::<String>(key::ID, &mut errs),
                o.req::<String>(key::PATH, &mut errs),
            );
            let mut limit = |name: &str, default: u64, errs: &mut Errs| -> Option<u64> {
                let value = o.opt::<u64>(name, errs).unwrap_or(default);
                if value == 0 {
                    errs.add(&o.at().key(name), Rule::BadValue, format!("`{name}` is at least 1"));
                    return None;
                }
                Some(value)
            };
            let max_files = limit(key::MAX_FILES, DEFAULT_MAX_FILES as u64, &mut errs);
            let max_file_bytes = limit(key::MAX_FILE_BYTES, DEFAULT_MAX_FILE_BYTES, &mut errs);
            let max_depth = limit(key::MAX_DEPTH, DEFAULT_MAX_DEPTH as u64, &mut errs);
            let timeout = limit(key::TIMEOUT, DEFAULT_TIMEOUT as u64, &mut errs);
            let max_iter = limit(key::MAX_ITER, DEFAULT_MAX_ITER as u64, &mut errs);
            let run_scripts = o.opt::<bool>(key::RUN_SCRIPTS, &mut errs).unwrap_or(true);
            if let (
                Some(id),
                Some(path),
                Some(max_files),
                Some(max_file_bytes),
                Some(max_depth),
                Some(timeout),
                Some(max_iter),
            ) = (id, path, max_files, max_file_bytes, max_depth, timeout, max_iter)
                && let Some(mut skill) = load(&at, id, path, &mut errs, &mut warnings)
            {
                (skill.max_files, skill.max_file_bytes, skill.max_depth) =
                    (max_files as usize, max_file_bytes, max_depth as usize);
                (skill.run_scripts, skill.timeout, skill.max_iter) = (run_scripts, timeout as u32, max_iter as u32);
                skills.push(skill);
            }
        }
        errs.finish(SkillsManager { skills, warnings })
    }

    /// What the build found worth saying without stopping.
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }

    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Skill> {
        self.skills.iter()
    }
}

/// Reads the SKILL.md of the folder and checks it against the standard.
fn load(at: &Path, id: String, path: String, errs: &mut Errs, warnings: &mut Vec<Warning>) -> Option<Skill> {
    let file = FsPath::new(&path).join(FILE);
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) => {
            errs.add(at, Rule::Io, format!("cannot read `{}`: {e}", file.display()));
            return None;
        }
    };
    let Some((front, _)) = split(&text) else {
        errs.add(
            at,
            Rule::BadValue,
            format!(
                "`{}` must start with a frontmatter between two `---` lines",
                file.display()
            ),
        );
        return None;
    };
    let front: Frontmatter = match serde_norway::from_str(front) {
        Ok(f) => f,
        Err(e) => {
            errs.add(
                at,
                Rule::BadValue,
                format!("the frontmatter of `{}` is not valid: {e}", file.display()),
            );
            return None;
        }
    };

    for field in front.other.keys() {
        warnings.push(Warning::new(
            Phase::Interpretation,
            at.clone(),
            "unknown_field",
            format!(
                "`{}` has the field `{field}`, which the Agent Skills standard does not define: ignored",
                file.display()
            ),
        ));
    }

    let before = errs.0.len();
    let name = front.name.unwrap_or_else(|| {
        errs.add(at, Rule::MissingField, "the frontmatter needs `name`");
        String::new()
    });
    if !name.is_empty() {
        check_name(at, &name, &path, errs);
    }
    let description = front.description.unwrap_or_else(|| {
        errs.add(at, Rule::MissingField, "the frontmatter needs `description`");
        String::new()
    });
    if !description.is_empty() && description.chars().count() > 1024 {
        errs.add(at, Rule::BadValue, "`description` is at most 1024 characters");
    }
    if let Some(c) = &front.compatibility
        && (c.is_empty() || c.chars().count() > 500)
    {
        errs.add(at, Rule::BadValue, "`compatibility` is from 1 to 500 characters");
    }
    if errs.0.len() != before {
        return None;
    }
    Some(Skill {
        id,
        path,
        name,
        description,
        license: front.license,
        compatibility: front.compatibility,
        metadata: front.metadata.unwrap_or_default(),
        allowed_tools: front.allowed_tools,
        max_files: DEFAULT_MAX_FILES,
        max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        max_depth: DEFAULT_MAX_DEPTH,
        run_scripts: true,
        timeout: DEFAULT_TIMEOUT,
        max_iter: DEFAULT_MAX_ITER,
    })
}

/// 1-64 characters: lowercase letters, numbers and hyphens; no hyphen at the start, at the end
/// or twice in a row; equal to the name of the folder.
fn check_name(at: &Path, name: &str, folder: &str, errs: &mut Errs) {
    let length = name.chars().count();
    if length > 64 {
        errs.add(at, Rule::BadValue, "`name` is at most 64 characters");
    }
    if !name
        .chars()
        .all(|c| c == '-' || c.is_ascii_digit() || (c.is_alphabetic() && c.is_lowercase()))
    {
        errs.add(
            at,
            Rule::BadValue,
            format!("`name` `{name}` may only have lowercase letters, numbers and hyphens"),
        );
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        errs.add(
            at,
            Rule::BadValue,
            format!("`name` `{name}` cannot start or end with a hyphen, nor have two in a row"),
        );
    }
    let dir = std::fs::canonicalize(folder)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
    if dir.as_deref() != Some(name) {
        errs.add(
            at,
            Rule::BadValue,
            format!(
                "`name` `{name}` must be the name of the folder `{}`",
                dir.unwrap_or_default()
            ),
        );
    }
}

/// The frontmatter and the body of a SKILL.md: the file starts with a `---` line and the
/// frontmatter ends at the next `---` line.
fn split(text: &str) -> Option<(&str, &str)> {
    let mut lines = text.split_inclusive('\n');
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    let start = text.find('\n')? + 1;
    let mut offset = start;
    for line in lines {
        if line.trim_end() == "---" {
            return Some((&text[start..offset], &text[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}
