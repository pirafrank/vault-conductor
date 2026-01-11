use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use signature::Signer;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{Extension, Identity, SignRequest};
use ssh_key::{PrivateKey, Signature};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Struct that holds both secret key and value
pub struct SecretData {
    pub name: String,
    pub value: String,
}

// 1. Define a trait for fetching secrets
#[async_trait]
pub trait SecretFetcher: Send + Sync + 'static {
    async fn get_secret(&self, id: Uuid) -> Result<SecretData>;
}

// 2. The Agent logic now relies on the trait, not the concrete Client
#[derive(Clone)]
pub struct BitwardenAgent<F: SecretFetcher + Clone> {
    fetcher: Arc<F>,
    secret_ids: Vec<Uuid>,
    cached_keys: Arc<Mutex<Vec<Option<PrivateKey>>>>,
    cached_key_names: Arc<Mutex<Vec<Option<String>>>>,
}

impl<F: SecretFetcher + Clone> BitwardenAgent<F> {
    pub fn new(fetcher: Arc<F>, secret_ids: Vec<Uuid>) -> Self {
        let count = secret_ids.len();
        Self {
            fetcher,
            secret_ids,
            cached_keys: Arc::new(Mutex::new(vec![None; count])),
            cached_key_names: Arc::new(Mutex::new(vec![None; count])),
        }
    }

    async fn get_private_key(&self, index: usize) -> Result<PrivateKey, AgentError> {
        // Check Cache
        {
            let cache = self.cached_keys.lock().unwrap();
            if let Some(Some(key)) = cache.get(index) {
                return Ok(key.clone());
            }
        }

        // Get the secret ID for this index
        let secret_id = self.secret_ids.get(index).ok_or_else(|| {
            AgentError::other(Box::new(std::io::Error::other("Invalid key index")))
        })?;

        // Fetch via Trait (gets both key and value in one call)
        let secret_data = self
            .fetcher
            .get_secret(*secret_id)
            .await
            .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?;

        // Parse
        let key = PrivateKey::from_openssh(&secret_data.value)
            .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?;

        // Update both caches
        let mut key_cache = self.cached_keys.lock().unwrap();
        if let Some(slot) = key_cache.get_mut(index) {
            *slot = Some(key.clone());
        }

        let mut name_cache = self.cached_key_names.lock().unwrap();
        if let Some(slot) = name_cache.get_mut(index) {
            *slot = Some(secret_data.name);
        }

        Ok(key)
    }

    fn get_cached_key_name(&self, index: usize) -> String {
        let cache = self.cached_key_names.lock().unwrap();
        // write a placeholder key name to be shown as key comment
        cache
            .get(index)
            .and_then(|opt| opt.clone())
            .unwrap_or_else(|| "bitwarden-sdk-key".to_string())
    }
}

#[async_trait]
impl<F: SecretFetcher + Clone + 'static> Session for BitwardenAgent<F> {
    async fn request_identities(&mut self) -> Result<Vec<Identity>, AgentError> {
        debug!("Request identities called");

        let mut identities = Vec::new();

        for index in 0..self.secret_ids.len() {
            let key = self.get_private_key(index).await?;
            let pubkey = key.public_key();

            // Log the public key details for debugging
            let key_data = pubkey.key_data();
            debug!(
                "Returning identity {} - algorithm: {:?}, fingerprint: {}",
                index,
                pubkey.algorithm(),
                pubkey.fingerprint(ssh_key::HashAlg::Sha256)
            );

            // Also log the key in authorized_keys format for comparison
            let auth_key_format = pubkey.to_openssh().unwrap_or_else(|_| "error".to_string());
            debug!("Public key {} (OpenSSH format): {}", index, auth_key_format);

            identities.push(Identity {
                pubkey: key_data.clone(),
                comment: self.get_cached_key_name(index),
            });
        }

        Ok(identities)
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {
        debug!(
            "Sign request - flags: 0x{:x}, data length: {} bytes",
            request.flags,
            request.data.len()
        );
        debug!(
            "Data (first 100 bytes): {:?}",
            &request.data[..request.data.len().min(100)]
        );

        // Find which key matches the requested public key
        for index in 0..self.secret_ids.len() {
            let key = self.get_private_key(index).await?;
            let pubkey = key.public_key();

            // Compare the public keys
            if pubkey.key_data() == &request.pubkey {
                // For SSH agent protocol, we need to create a RAW signature (not OpenSSH format)
                // using the underlying keypair's try_sign method
                let signature_bytes = key.try_sign(&request.data).map_err(|e| {
                    AgentError::other(Box::new(std::io::Error::other(format!(
                        "Signing failed: {}",
                        e
                    ))))
                })?;

                debug!(
                    "Signature created successfully with key {}, {} bytes",
                    index,
                    signature_bytes.as_bytes().len()
                );

                // Return the signature in SSH agent format
                return Ok(signature_bytes);
            }
        }

        // No matching key found
        Err(AgentError::other(Box::new(std::io::Error::other(
            "Key not found",
        ))))
    }

    async fn extension(&mut self, extension: Extension) -> Result<Option<Extension>, AgentError> {
        debug!("Extension request: {}", extension.name);

        // Return None to indicate the extension is not supported but don't error
        // This allows clients to gracefully handle unsupported extensions
        Ok(None)
    }
}
