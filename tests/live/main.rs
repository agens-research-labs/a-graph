//! Tests of the topic `live`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "blackboard/blackboard.rs"]
mod blackboard;

#[path = "image/image.rs"]
mod image;

#[path = "md_media/md_media.rs"]
mod md_media;

#[path = "parallel_group/parallel_group.rs"]
mod parallel_group;

#[path = "plain_answer/plain_answer.rs"]
mod plain_answer;

#[path = "react/react.rs"]
mod react;

#[path = "reasoning/reasoning.rs"]
mod reasoning;

#[path = "schema_json/schema_json.rs"]
mod schema_json;

#[path = "sequence/sequence.rs"]
mod sequence;
