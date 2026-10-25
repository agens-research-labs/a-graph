# Skills

A skill is a folder with a `SKILL.md`, in the [Agent Skills](https://agentskills.io/specification) format: written
instructions for a task, and the files and scripts they need. The model decides at run time whether to use one.
Skills are declared in `resources.skills` and a node lists the ones it uses in `resources.skills`. Every key is
described in [the reference](graph.md).

```yaml
resources:
  skills:
    - id: pdf
      path: ./skills/pdf-processing
```

## The folder

`SKILL.md` starts with a frontmatter between two `---` lines, followed by the instructions in Markdown. The file is
read when the graph file is checked, and it must follow the standard:

- `name` (mandatory): at most 64 characters, only lowercase letters, digits and hyphens, not starting or ending with
  a hyphen and without two in a row; it must be the name of the folder;
- `description` (mandatory): at most 1024 characters;
- `compatibility` (optional): from 1 to 500 characters;
- `license`, `metadata` and `allowed-tools` (optional).

A field that the standard does not define is ignored, with a warning. The name and the description are written only
in `SKILL.md`, never in the graph file.

`allowed-tools` (the "pre-approved tools" of the standard, still experimental there) is read and kept, and nothing
more: it is a permission to skip the confirmation of the user, and a-graph never asks for one, so every tool of the
node is already approved. It is not a restriction, and nothing is checked against it.

## How the model uses a skill

The instructions are given only when the model asks for them:

1. The system text of the node gets, at its end, how to use the skills and a catalog with the `name` and the
   `description` of each skill of the node.
2. The model activates a skill by calling `activate_skill`, a tool of the program whose only parameter is the name of
   the skill. The answer is the instructions of `SKILL.md`, without the frontmatter, inside
   `<skill_content name="...">`. It also lists the other files of the folder, with paths relative to the folder: hidden
   files and links that lead out of the folder are left out, at most `max_files` files are listed (a longer list says
   that it is incomplete) and at most `max_depth` levels of sub-folders are looked into. The files are listed, not
   read. A skill is given once in a run of the node: a second activation is told that it is already loaded.
3. The model reads a file with `read_skill_file`, a second tool of the program, with the parameters `skill` and `path`
   (relative to the folder of the skill). Only text files inside the folder, of at most `max_file_bytes` bytes, can be
   read. An absolute path, a `..`, a link that leads out, and a missing, binary or too large file are told to the model
   as errors it can read.
4. The model runs a script with `run_skill_script`, a third tool of the program, with `skill`, `path` (a file inside
   the folder `scripts/` of the skill) and `args` (a list of texts). This tool exists only for the skills with
   `run_scripts: true`, which is the default.

## Scripts

A script is started as it is, without a shell, so nothing in its arguments is expanded. It runs in the folder of the
skill, with no standard input, and its environment holds only `PATH`: it does not get the keys of the models. It must
have the permission to execute, and a `#!` line if it is a script.

The result is what it wrote on its standard output, cut at `max_file_bytes` bytes. An exit code other than 0, a
missing permission and a run longer than `timeout` seconds reach the model as errors.

## Limits

- `max_iter` is the most calls of `read_skill_file` and `run_skill_script` together, for one skill, in one run of a
  node. Asking once more fails the run (`tool_limit`). The activation is not counted.
- `activate_skill`, `read_skill_file` and `run_skill_script` are reserved: a node with skills cannot have a tool with one
  of those names, nor two skills with the same name (`duplicate_id`).
- Every limit (`max_files`, `max_file_bytes`, `max_depth`, `timeout`, `max_iter`) is at least 1.
