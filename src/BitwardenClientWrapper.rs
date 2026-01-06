use anyhow::{Context, Result, anyhow};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{secrets::SecretGetRequest, ClientSecretsExt},
    Client,
};
use std::sync::Arc;
use tokio::net::UnixListener;
use uuid::Uuid;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;

// Import from our lib
use crate::lib::{BitwardenAgent, SecretFetcher}; 

// Real implementation wrapper
struct BitwardenClientWrapper(Client);

#[async_trait::async_trait]
impl SecretFetcher for BitwardenClientWrapper {
    async fn get_secret_value(&self, id: Uuid) -> Result<String> {
        let request = SecretGetRequest { id };
        let response = self.0.secrets().get(&request).await
            .map_err(|e| anyhow!("Bitwarden SDK Error: {}", e))?;
        Ok(response.value)
    }
}

pub async fn start_agent() -> Result<()> {
    // ... (Same setup code as before: env vars, socket paths) ...
    // Note: Omitted for brevity, but identical to previous answer except for:

    let access_token = std::env::var("BWS_ACCESS_TOKEN").context("Token required")?;
    let secret_id_str = std::env::var("BW_SECRET_ID").context("ID required")?;
    let secret_id = Uuid::parse_str(&secret_id_str)?;

    let mut client = Client::new(None);
    client.auth().login_access_token(&AccessTokenLoginRequest { access_token, state_file: None }).await.map_err(|e| anyhow!(e))?;
    
    // Wrap the client in our Trait implementation
    let fetcher = Arc::new(BitwardenClientWrapper(client));

    // ... (Socket binding code) ...
    let listener = UnixListener::bind("/tmp/agent.sock")?;

    loop {
        let (socket, _) = listener.accept().await?;
        let agent = BitwardenAgent::new(fetcher.clone(), secret_id);
        tokio::spawn(async move {
            let _ = ssh_agent_lib::agent::process_session(socket, agent).await;
        });
    }
}
