//! Tests of the topic `schema`: the JSON Schemas and the references generated from the code, of the
//! graph and of the file of the service, and the description of the API of the service, are the
//! files in the repository, and every key has its description.

#[path = "api_up_to_date/api_up_to_date.rs"]
mod api_up_to_date;

#[path = "all_documented/all_documented.rs"]
mod all_documented;

#[path = "docs_up_to_date/docs_up_to_date.rs"]
mod docs_up_to_date;

#[path = "service_docs_up_to_date/service_docs_up_to_date.rs"]
mod service_docs_up_to_date;

#[path = "service_up_to_date/service_up_to_date.rs"]
mod service_up_to_date;

#[path = "up_to_date/up_to_date.rs"]
mod up_to_date;
