use anyhow::{anyhow, Context, Result};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{secrets::SecretGetRequest, ClientSecretsExt},
    Client,
};
use std::sync::Arc;

#[cfg(not(windows))]
use tokio::net::UnixListener as Listener;

use uuid::Uuid;

// Import from our lib
use crate::bitwarden::agent::{BitwardenAgent, SecretFetcher};

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
        let response = self
            .0
            .secrets()
            .get(&request)
            .await
            .map_err(|e| anyhow!("Bitwarden SDK Error: {}", e))?;
        Ok(response.value)
    }
}

pub async fn start_agent() -> Result<()> {
    let access_token = std::env::var("BWS_ACCESS_TOKEN")
        .context("Token required in BWS_ACCESS_TOKEN environment variable")?;
    let secret_id_str = std::env::var("BW_SECRET_ID")
        .context("ID required in BW_SECRET_ID environment variable")?;
    let secret_id = Uuid::parse_str(&secret_id_str)?;

    let client = Client::new(None);
    client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token,
            state_file: None,
        })
        .await
        .map_err(|e| anyhow!(e))?;

    // Wrap the client in our Trait implementation
    let fetcher = Arc::new(BitwardenClientWrapper(Arc::new(client)));

    // Remove existing socket if it exists
    #[cfg(not(windows))]
    let _ = std::fs::remove_file(SOCKET_NAME);

    let listener = Listener::bind(SOCKET_NAME)?;

    // Use ssh-agent-lib's listen function with a Session implementation
    use ssh_agent_lib::agent::listen;

    // Create the agent instance
    let agent = BitwardenAgent::new(fetcher.clone(), secret_id);

    // Listen and process connections
    listen(listener, agent).await?;

    Ok(())
}
