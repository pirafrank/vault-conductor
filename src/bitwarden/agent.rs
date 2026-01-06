use anyhow::Result;
use async_trait::async_trait;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{Identity, SignRequest};
use ssh_key::{HashAlg, PrivateKey, Signature};
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
        let key = self.get_private_key().await?;
        let pubkey = key.public_key();
        Ok(vec![Identity {
            pubkey: pubkey.key_data().clone(),
            comment: "bitwarden-sdk-key".to_string(),
        }])
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {
        let key = self.get_private_key().await?;
        let pubkey = key.public_key();

        // Compare the public keys
        if pubkey.key_data() != &request.pubkey {
            return Err(AgentError::other(Box::new(std::io::Error::other(
                "Key not found",
            ))));
        }

        // Determine the hash algorithm based on the key type
        // For SSH agent protocol, we typically use SHA256 or SHA512
        let hash_alg = match pubkey.key_data() {
            ssh_key::public::KeyData::Ed25519(_) => HashAlg::Sha512,
            ssh_key::public::KeyData::Rsa(_) => HashAlg::Sha512,
            _ => HashAlg::Sha256,
        };

        // Sign the data using the private key
        // For SSH agent protocol, we use namespace="" for standard SSH signatures
        let ssh_sig = key
            .sign("", hash_alg, &request.data)
            .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?;

        // Convert SshSig to Signature
        Ok(Signature::new(
            ssh_sig.algorithm().clone(),
            ssh_sig.signature_bytes().to_vec(),
        )
        .map_err(|e| AgentError::other(Box::new(std::io::Error::other(e.to_string()))))?)
    }
}
