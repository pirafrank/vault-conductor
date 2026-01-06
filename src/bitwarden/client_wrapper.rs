use anyhow::{anyhow, Context, Result};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{secrets::SecretGetRequest, ClientSecretsExt},
    Client,
};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

#[cfg(not(windows))]
use tokio::net::UnixListener as Listener;

use uuid::Uuid;

// Import from our lib
use crate::bitwarden::agent::{BitwardenAgent, SecretFetcher};
use crate::config::{Config, CONFIG_FILE};

// Socket setup
#[cfg(not(windows))]
const SOCKET_NAME: &str = "/tmp/vc-ssh-agent.sock";

// Real implementation wrapper - needs to be Clone
#[derive(Clone)]
pub struct BitwardenClientWrapper(Arc<Client>);

#[async_trait::async_trait]
impl SecretFetcher for BitwardenClientWrapper {
    async fn get_secret_value(&self, id: Uuid) -> Result<String> {
        let request = SecretGetRequest { id };
        let response = self.0.secrets().get(&request).await.map_err(|e| {
            anyhow!(
                "Bitwarden SDK: Failed to fetch secret '{}'.\nError: {}",
                id,
                e
            )
        })?;
        Ok(response.value)
    }
}

pub async fn start_agent_foreground() -> Result<()> {
    let config =
        Config::load().context(format!("Failed to load configuration from {}", CONFIG_FILE))?;

    let secret_id = Uuid::parse_str(&config.bw_secret_id)?;

    let client = Client::new(None);
    client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token: config.bws_access_token.clone(),
            state_file: None,
        })
        .await
        .map_err(|e| {
            anyhow!(
                "Bitwarden SDK: Authentication failed.\nPlease check your access token. \
                The token may be invalid, expired, or from an incompatible SDK version.\nError: {}",
                e
            )
        })?;

    // Wrap the client in our Trait implementation
    let fetcher = Arc::new(BitwardenClientWrapper(Arc::new(client)));

    // Remove existing socket if it exists
    #[cfg(not(windows))]
    let _ = std::fs::remove_file(SOCKET_NAME);

    let listener = Listener::bind(SOCKET_NAME)?;
    // Set socket permissions to 0600 (read/write for owner only)
    std::fs::set_permissions(SOCKET_NAME, std::fs::Permissions::from_mode(0o600))
        .context("Failed to set socket permissions")?;

    // Use ssh-agent-lib's listen function with a Session implementation
    use ssh_agent_lib::agent::listen;

    // Create the agent instance
    let agent = BitwardenAgent::new(fetcher.clone(), secret_id);

    // Listen and process connections
    listen(listener, agent).await?;

    Ok(())
}
