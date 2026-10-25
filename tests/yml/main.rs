//! Tests of the topic `yml`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "all_errors/all_errors.rs"]
mod all_errors;

#[path = "duplicate_id/duplicate_id.rs"]
mod duplicate_id;

#[path = "missing_block/missing_block.rs"]
mod missing_block;

#[path = "misspelled_key/misspelled_key.rs"]
mod misspelled_key;

#[path = "other_provider_key/other_provider_key.rs"]
mod other_provider_key;

#[path = "parse_error/parse_error.rs"]
mod parse_error;

#[path = "id_unique_across_kinds/id_unique_across_kinds.rs"]
mod id_unique_across_kinds;

#[path = "secret_env_ref/secret_env_ref.rs"]
mod secret_env_ref;

#[path = "unknown_key/unknown_key.rs"]
mod unknown_key;

#[path = "unknown_provider/unknown_provider.rs"]
mod unknown_provider;

#[path = "unknown_ref/unknown_ref.rs"]
mod unknown_ref;

#[path = "valid_loads/valid_loads.rs"]
mod valid_loads;

#[path = "word_not_in_list/word_not_in_list.rs"]
mod word_not_in_list;

#[path = "wrong_type/wrong_type.rs"]
mod wrong_type;
