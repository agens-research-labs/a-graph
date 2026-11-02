//! The hosts Bedrock is reached at, for the network allow-list. They are resolved with the
//! endpoint rules of the SDK itself, with the same settings the clients are pinned to
//! (`pinned`): no FIPS, no dual-stack, no endpoint from the environment or the AWS profile. So
//! the hosts checked are the hosts called.

use aws_config::SdkConfig;
use aws_smithy_types::endpoint::Endpoint;
use aws_types::service_config::{LoadServiceConfig, ServiceConfigKey};
use futures_util::FutureExt;

use crate::net::host_of;

/// `s3://bucket/some/prefix/` -> (`bucket`, `some/prefix/`). The prefix ends with `/` or is empty.
pub(super) fn split_s3(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix("s3://")?;
    let (bucket, prefix) = rest.split_once('/').unwrap_or((rest, ""));
    if bucket.is_empty() {
        return None;
    }
    let prefix = if prefix.is_empty() || prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };
    Some((bucket.to_string(), prefix))
}

/// The SDK configuration the clients are built from, with only the settings the hosts are
/// resolved with. `endpoint_url` is a server of the tests, never from the yml.
pub(super) fn pinned(sdk: SdkConfig, endpoint_url: Option<String>) -> SdkConfig {
    let mut b = sdk.into_builder();
    b.set_use_fips(Some(false));
    b.set_use_dual_stack(Some(false));
    b.set_endpoint_url(endpoint_url);
    // the per-service endpoints of the environment (`AWS_ENDPOINT_URL_S3`, ...) live here
    b.set_service_config(None::<NoServiceConfig>);
    b.build()
}

/// The type of an absent service configuration.
#[derive(Debug)]
struct NoServiceConfig;

impl LoadServiceConfig for NoServiceConfig {
    fn load_config(&self, _key: ServiceConfigKey<'_>) -> Option<String> {
        None
    }
}

/// The host of an endpoint the SDK resolved. Its default resolvers are synchronous.
fn host<E: std::fmt::Display>(resolved: Option<Result<Endpoint, E>>, what: &str) -> Result<String, String> {
    let endpoint = resolved
        .ok_or_else(|| format!("the endpoint of {what} cannot be resolved now"))?
        .map_err(|e| format!("no endpoint for {what}: {e}"))?;
    host_of(endpoint.url()).ok_or_else(|| format!("the endpoint of {what} has no host"))
}

/// The Converse API, the calls of every node.
pub(crate) fn runtime_host(region: &str) -> Result<String, String> {
    use aws_sdk_bedrockruntime::config::endpoint::{DefaultResolver, Params, ResolveEndpoint};
    let params = Params::builder()
        .region(region)
        .use_fips(false)
        .use_dual_stack(false)
        .build()
        .map_err(|e| e.to_string())?;
    host(
        DefaultResolver::new().resolve_endpoint(&params).now_or_never(),
        &format!("Bedrock in region `{region}`"),
    )
}

/// The control API, where a batch job is created and followed.
pub(crate) fn control_host(region: &str) -> Result<String, String> {
    use aws_sdk_bedrock::config::endpoint::{DefaultResolver, Params, ResolveEndpoint};
    let params = Params::builder()
        .region(region)
        .use_fips(false)
        .use_dual_stack(false)
        .build()
        .map_err(|e| e.to_string())?;
    host(
        DefaultResolver::new().resolve_endpoint(&params).now_or_never(),
        &format!("the Bedrock control API in region `{region}`"),
    )
}

/// The S3 bucket of an `s3://` URI of a batch, reached from `region`.
pub(crate) fn s3_host(uri: &str, region: &str) -> Result<String, String> {
    use aws_sdk_s3::config::endpoint::{DefaultResolver, Params, ResolveEndpoint};
    let (bucket, _) = split_s3(uri).ok_or_else(|| format!("`{uri}` is not an s3:// URI"))?;
    let params = Params::builder()
        .bucket(&bucket)
        .region(region)
        .use_fips(false)
        .use_dual_stack(false)
        .build()
        .map_err(|e| e.to_string())?;
    host(
        DefaultResolver::new().resolve_endpoint(&params).now_or_never(),
        &format!("the S3 bucket `{bucket}` in region `{region}`"),
    )
}
