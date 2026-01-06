use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use ssh_agent_lib::agent::{Session, Error as AgentError};
use ssh_agent_lib::proto::{Identity, SignRequest};
use ssh_key::{PrivateKey, Signature};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// 1. Define a trait for fetching secrets
#[async_trait]
pub trait SecretFetcher: Send + Sync {
    async fn get_secret_value(&self, id: Uuid) -> Result<String>;
}

// 2. The Agent logic now relies on the trait, not the concrete Client
pub struct BitwardenAgent<F: SecretFetcher> {
    fetcher: Arc<F>,
    secret_id: Uuid,
    cached_key: Arc<Mutex<Option<PrivateKey>>>,
}

impl<F: SecretFetcher> BitwardenAgent<F> {
    pub fn new(fetcher: Arc<F>, secret_id: Uuid) -> Self {
        Self {
            fetcher,
            secret_id,
            cached_key: Arc::new(Mutex::new(None)),
        }
    }

    async fn get_private_key(&self) -> Result<PrivateKey, AgentError> {
        // Check Cache
        {
            let cache = self.cached_key.lock().unwrap();
            if let Some(key) = &*cache {
                return Ok(key.clone());
            }
        }

        // Fetch via Trait
        let key_pem = self.fetcher.get_secret_value(self.secret_id).await
            .map_err(|e| AgentError::from(format!("Fetch Error: {}", e)))?;

        // Parse
        let key = PrivateKey::from_openssh(&key_pem)
            .map_err(|e| AgentError::from(format!("Bad key format: {}", e)))?;

        // Update Cache
        let mut cache = self.cached_key.lock().unwrap();
        *cache = Some(key.clone());
        
        Ok(key)
    }
}

#[async_trait]
impl<F: SecretFetcher> Session for BitwardenAgent<F> {
    async fn request_identities(&mut self) -> Result<Vec<Identity>, AgentError> {
        let key = self.get_private_key().await?;
        Ok(vec![Identity {
            pubkey_blob: key.public_key().to_bytes().map_err(|e| AgentError::from(e.to_string()))?,
            comment: "bitwarden-sdk-key".to_string(),
        }])
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {
        let key = self.get_private_key().await?;
        let pubkey = key.public_key();

        if pubkey.to_bytes().map_err(|_| AgentError::from("Key encoding error"))? != request.pubkey_blob {
             return Err(AgentError::from("Key not found"));
        }

        key.sign(request.data.as_slice())
            .map_err(|e| AgentError::from(format!("Signing failed: {}", e)))
    }
}
