//! Tests of the topic `media`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "compression_rules/compression_rules.rs"]
mod compression_rules;

#[path = "gemini_size_512/gemini_size_512.rs"]
mod gemini_size_512;

#[path = "gemini_size_unknown/gemini_size_unknown.rs"]
mod gemini_size_unknown;

#[path = "image_base64/image_base64.rs"]
mod image_base64;

#[path = "media_base64/media_base64.rs"]
mod media_base64;

#[path = "media_family/media_family.rs"]
mod media_family;

#[path = "media_from_node/media_from_node.rs"]
mod media_from_node;

#[path = "media_input_generated/media_input_generated.rs"]
mod media_input_generated;

#[path = "media_markdown/media_markdown.rs"]
mod media_markdown;

#[path = "media_one_writer/media_one_writer.rs"]
mod media_one_writer;

#[path = "media_source_or_generate/media_source_or_generate.rs"]
mod media_source_or_generate;

#[path = "provider_needs_generate/provider_needs_generate.rs"]
mod provider_needs_generate;

#[path = "style_vs_moderation/style_vs_moderation.rs"]
mod style_vs_moderation;

#[path = "transparency_rules/transparency_rules.rs"]
mod transparency_rules;

#[path = "two_families/two_families.rs"]
mod two_families;

#[path = "url_gzip_budget/url_gzip_budget.rs"]
mod url_gzip_budget;

#[path = "url_gzip_read/url_gzip_read.rs"]
mod url_gzip_read;

#[path = "url_md_both/url_md_both.rs"]
mod url_md_both;

#[path = "url_md_read/url_md_read.rs"]
mod url_md_read;

#[path = "url_refused/url_refused.rs"]
mod url_refused;

#[path = "url_too_slow/url_too_slow.rs"]
mod url_too_slow;

#[path = "url_unused_skipped/url_unused_skipped.rs"]
mod url_unused_skipped;
