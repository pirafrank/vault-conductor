use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use signature::Signer;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{Extension, Identity, SignRequest};
use ssh_key::{PrivateKey, Signature};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// 1. Define a trait for fetching secrets
#[async_trait]
pub trait SecretFetcher: Send + Sync + 'static {
    async fn get_secret_value(&self, id: Uuid) -> Result<String>;
}

// 2. The Agent logic now relies on the trait, not the concrete Client
#[derive(Clone)]
pub struct BitwardenAgent<F: SecretFetcher + Clone> {
    fetcher: Arc<F>,
    secret_id: Uuid,
    cached_key: Arc<Mutex<Option<PrivateKey>>>,
}

impl<F: SecretFetcher + Clone> BitwardenAgent<F> {
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
        let key_pem = self
            .fetcher
            .get_secret_value(self.secret_id)
            .await
            .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?;

        // Parse
        let key = PrivateKey::from_openssh(&key_pem)
            .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?;

        // Update Cache
        let mut cache = self.cached_key.lock().unwrap();
        *cache = Some(key.clone());

        Ok(key)
    }
}

#[async_trait]
impl<F: SecretFetcher + Clone + 'static> Session for BitwardenAgent<F> {
    async fn request_identities(&mut self) -> Result<Vec<Identity>, AgentError> {
        info!("Request identities called");
        let key = self.get_private_key().await?;
        let pubkey = key.public_key();

        // Log the public key details for debugging
        let key_data = pubkey.key_data();
        info!(
            "Returning identity - algorithm: {:?}, fingerprint: {}",
            pubkey.algorithm(),
            pubkey.fingerprint(ssh_key::HashAlg::Sha256)
        );

        // Also log the key in authorized_keys format for comparison
        let auth_key_format = pubkey.to_openssh().unwrap_or_else(|_| "error".to_string());
        info!("Public key (OpenSSH format): {}", auth_key_format);

        Ok(vec![Identity {
            pubkey: key_data.clone(),
            comment: "bitwarden-sdk-key".to_string(),
        }])
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {
        info!(
            "Sign request - flags: 0x{:x}, data length: {} bytes",
            request.flags,
            request.data.len()
        );
        debug!(
            "Data (first 100 bytes): {:?}",
            &request.data[..request.data.len().min(100)]
        );

        let key = self.get_private_key().await?;
        let pubkey = key.public_key();

        // Compare the public keys
        if pubkey.key_data() != &request.pubkey {
            return Err(AgentError::other(Box::new(std::io::Error::other(
                "Key not found",
            ))));
        }

        // For SSH agent protocol, we need to create a RAW signature (not OpenSSH format)
        // using the underlying keypair's try_sign method
        let signature_bytes = key.try_sign(&request.data).map_err(|e| {
            AgentError::other(Box::new(std::io::Error::other(format!(
                "Signing failed: {}",
                e
            ))))
        })?;

        info!(
            "Signature created successfully, {} bytes",
            signature_bytes.as_bytes().len()
        );

        // Return the signature in SSH agent format
        Ok(signature_bytes)
    }

    async fn extension(&mut self, extension: Extension) -> Result<Option<Extension>, AgentError> {
        info!("Extension request: {}", extension.name);

        // Return None to indicate the extension is not supported but don't error
        // This allows clients to gracefully handle unsupported extensions
        Ok(None)
    }
}
