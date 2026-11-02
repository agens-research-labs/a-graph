//! Ambient credentials: where a bearer token comes from when the yml names no key.
//!
//! They are resolved at the first call, never at load (the traffic they make is outside the
//! allow-list, as the yml says). A `TokenSource` is also what tests plug in instead.

use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::OnceCell;

/// Gives a bearer token. The error is a plain description: the caller wraps it in a `ModelError`.
#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn token(&self) -> Result<String, String>;
}

/// A token that never changes (tests, or a token the caller already has).
pub struct StaticToken(pub String);

#[async_trait]
impl TokenSource for StaticToken {
    async fn token(&self) -> Result<String, String> {
        Ok(self.0.clone())
    }
}

const GOOGLE_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// Google Application Default Credentials: `GOOGLE_APPLICATION_CREDENTIALS`, then the gcloud
/// file, then the metadata server (the library's own order).
#[derive(Default)]
pub struct GoogleAdc {
    credentials: OnceCell<google_cloud_auth::credentials::AccessTokenCredentials>,
}

impl GoogleAdc {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TokenSource for GoogleAdc {
    async fn token(&self) -> Result<String, String> {
        let credentials = self
            .credentials
            .get_or_try_init(|| async {
                google_cloud_auth::credentials::Builder::default()
                    .with_scopes([GOOGLE_SCOPE])
                    .build_access_token_credentials()
                    .map_err(|e| format!("no Google credentials found: {e}"))
            })
            .await?;
        let token = credentials
            .access_token()
            .await
            .map_err(|e| format!("cannot get a Google access token: {e}"))?;
        Ok(token.token)
    }
}

/// How long the managed identity gets before the developer tools are tried: outside Azure it
/// has nothing to answer.
const MANAGED_IDENTITY_WAIT: Duration = Duration::from_secs(3);

/// Azure ambient credentials, tried in order: the managed identity (an app running on an Azure
/// resource), then the developer tools (Azure CLI, then Azure Developer CLI).
pub struct AzureAmbient {
    scope: &'static str,
}

impl AzureAmbient {
    /// `scope`: what the token is for (it differs between the Foundry endpoints).
    pub fn new(scope: &'static str) -> Self {
        AzureAmbient { scope }
    }
}

#[async_trait]
impl TokenSource for AzureAmbient {
    async fn token(&self) -> Result<String, String> {
        use azure_core::credentials::TokenCredential;

        let scopes = [self.scope];
        let mut why = Vec::new();

        match azure_identity::ManagedIdentityCredential::new(None) {
            Ok(managed) => match tokio::time::timeout(MANAGED_IDENTITY_WAIT, managed.get_token(&scopes, None)).await {
                Ok(Ok(t)) => return Ok(t.token.secret().to_string()),
                Ok(Err(e)) => why.push(format!("managed identity: {e}")),
                Err(_) => why.push("managed identity: no answer".to_string()),
            },
            Err(e) => why.push(format!("managed identity: {e}")),
        }
        match azure_identity::DeveloperToolsCredential::new(None) {
            Ok(tools) => match tools.get_token(&scopes, None).await {
                Ok(t) => return Ok(t.token.secret().to_string()),
                Err(e) => why.push(format!("developer tools (Azure CLI, Azure Developer CLI): {e}")),
            },
            Err(e) => why.push(format!("developer tools: {e}")),
        }
        Err(format!("no Azure credentials found ({})", why.join("; ")))
    }
}
