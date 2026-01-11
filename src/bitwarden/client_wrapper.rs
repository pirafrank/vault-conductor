use crate::file_manager::{cleanup_files, get_socket_file_path, remove_file};
use anyhow::{anyhow, Context, Result};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{secrets::SecretGetRequest, ClientSecretsExt},
    Client,
};
use log::info;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

#[cfg(not(windows))]
use tokio::net::UnixListener as Listener;

use uuid::Uuid;

// Import from our lib
use crate::bitwarden::agent::{BitwardenAgent, SecretFetcher};
use crate::config::{Config, CONFIG_FILE};

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

pub async fn start_agent_foreground(config_file: Option<String>) -> Result<()> {
    let socket_path = get_socket_file_path();
    // Remove existing socket if it exists
    remove_file(&socket_path, "socket")?;
    // Load configuration
    let config = Config::load(config_file)
        .context(format!("Failed to load configuration from {}", CONFIG_FILE))?;

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

    let listener = Listener::bind(&socket_path)?;
    // Set socket permissions to 0600 (read/write for owner only)
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .context("Failed to set socket permissions")?;

    // Use ssh-agent-lib's listen function with a Session implementation
    use ssh_agent_lib::agent::listen;

    // Create the agent instance
    let agent = BitwardenAgent::new(fetcher.clone(), secret_id);

    // Setup signal handlers for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    // Listen and process connections with signal handling
    tokio::select! {
        result = listen(listener, agent) => {
            // Agent finished (unlikely in normal operation)
            if let Err(e) = result {
                cleanup_files()?;
                return Err(e.into());
            }
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, gracefully shutting down...");
            cleanup_files()?;
        }
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C), gracefully shutting down...");
            cleanup_files()?;
        }
    }

    Ok(())
}
