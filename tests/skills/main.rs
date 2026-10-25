//! Tests of the topic `skills`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "big_file_refused/big_file_refused.rs"]
mod big_file_refused;

#[path = "catalog_activation/catalog_activation.rs"]
mod catalog_activation;

#[path = "depth_zero/depth_zero.rs"]
mod depth_zero;

#[path = "distinct_skill_names/distinct_skill_names.rs"]
mod distinct_skill_names;

#[path = "extra_fields_warn/extra_fields_warn.rs"]
mod extra_fields_warn;

#[path = "file_limits_defaults/file_limits_defaults.rs"]
mod file_limits_defaults;

#[path = "file_limits_min/file_limits_min.rs"]
mod file_limits_min;

#[path = "file_list_cut/file_list_cut.rs"]
mod file_list_cut;

#[path = "files_listed/files_listed.rs"]
mod files_listed;

#[path = "instructions_on_demand/instructions_on_demand.rs"]
mod instructions_on_demand;

#[path = "keeps_metadata/keeps_metadata.rs"]
mod keeps_metadata;

#[path = "metadata_text_only/metadata_text_only.rs"]
mod metadata_text_only;

#[path = "name_description_req/name_description_req.rs"]
mod name_description_req;

#[path = "name_is_folder/name_is_folder.rs"]
mod name_is_folder;

#[path = "name_rules/name_rules.rs"]
mod name_rules;

#[path = "no_frontmatter/no_frontmatter.rs"]
mod no_frontmatter;

#[path = "no_skill_md/no_skill_md.rs"]
mod no_skill_md;

#[path = "path_leaves_folder/path_leaves_folder.rs"]
mod path_leaves_folder;

#[path = "read_file/read_file.rs"]
mod read_file;

#[path = "read_missing_binary/read_missing_binary.rs"]
mod read_missing_binary;

#[path = "read_unknown_skill/read_unknown_skill.rs"]
mod read_unknown_skill;

#[path = "run_script/run_script.rs"]
mod run_script;

#[path = "run_scripts_false/run_scripts_false.rs"]
mod run_scripts_false;

#[path = "script_args_cwd/script_args_cwd.rs"]
mod script_args_cwd;

#[path = "script_clean_env/script_clean_env.rs"]
mod script_clean_env;

#[path = "script_fails/script_fails.rs"]
mod script_fails;

#[path = "script_no_exec/script_no_exec.rs"]
mod script_no_exec;

#[path = "script_timeout/script_timeout.rs"]
mod script_timeout;

#[path = "script_unknown_skill/script_unknown_skill.rs"]
mod script_unknown_skill;

#[path = "scripts_defaults/scripts_defaults.rs"]
mod scripts_defaults;

#[path = "scripts_folder_only/scripts_folder_only.rs"]
mod scripts_folder_only;

#[path = "skill_given_once/skill_given_once.rs"]
mod skill_given_once;

#[path = "skill_max_iter/skill_max_iter.rs"]
mod skill_max_iter;

#[path = "text_length_limits/text_length_limits.rs"]
mod text_length_limits;

#[path = "unknown_skill/unknown_skill.rs"]
mod unknown_skill;
