//! Tests of the topic `nodes`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "absent_booleans_false/absent_booleans_false.rs"]
mod absent_booleans_false;

#[path = "entity_components/entity_components.rs"]
mod entity_components;

#[path = "external_board_parts/external_board_parts.rs"]
mod external_board_parts;

#[path = "missing_list/missing_list.rs"]
mod missing_list;

#[path = "model_component/model_component.rs"]
mod model_component;

#[path = "model_error_named/model_error_named.rs"]
mod model_error_named;

#[path = "schema_gives_json/schema_gives_json.rs"]
mod schema_gives_json;

#[path = "schema_not_object/schema_not_object.rs"]
mod schema_not_object;

#[path = "schema_yml_to_json/schema_yml_to_json.rs"]
mod schema_yml_to_json;

#[path = "to_node_no_reasoning/to_node_no_reasoning.rs"]
mod to_node_no_reasoning;

#[path = "to_output_reasoning/to_output_reasoning.rs"]
mod to_output_reasoning;

#[path = "provider_text_kept_out/provider_text_kept_out.rs"]
mod provider_text_kept_out;
