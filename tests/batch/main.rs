//! Tests of the topic `batch`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "batch_builds/batch_builds.rs"]
mod batch_builds;

#[path = "bedrock_one_error/bedrock_one_error.rs"]
mod bedrock_one_error;

#[path = "creating_cancels/creating_cancels.rs"]
mod creating_cancels;

#[path = "failed_branch_cancels/failed_branch_cancels.rs"]
mod failed_branch_cancels;

#[path = "group_in_batch/group_in_batch.rs"]
mod group_in_batch;

#[path = "limit_per_job/limit_per_job.rs"]
mod limit_per_job;

#[path = "model_no_batch/model_no_batch.rs"]
mod model_no_batch;

#[path = "one_job/one_job.rs"]
mod one_job;

#[path = "schema_after_job/schema_after_job.rs"]
mod schema_after_job;

#[path = "poll_fails_cancels/poll_fails_cancels.rs"]
mod poll_fails_cancels;

#[path = "preparing_no_job/preparing_no_job.rs"]
mod preparing_no_job;

#[path = "request_refused/request_refused.rs"]
mod request_refused;

#[path = "same_model/same_model.rs"]
mod same_model;

#[path = "stopped_cancels/stopped_cancels.rs"]
mod stopped_cancels;

#[path = "tools_refused/tools_refused.rs"]
mod tools_refused;

#[path = "logprobs_missing/logprobs_missing.rs"]
mod logprobs_missing;

#[path = "schema_prompt_refused/schema_prompt_refused.rs"]
mod schema_prompt_refused;
