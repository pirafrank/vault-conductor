use anyhow::Result;
use async_trait::async_trait;
use log::{debug, warn};
use signature::Signer;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{Extension, Identity, SignRequest};
use ssh_key::{PrivateKey, Signature};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Struct that holds both secret key and value
#[derive(Clone)]
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
            match self.get_private_key(index).await {
                Ok(key) => {
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
                    let auth_key_format =
                        pubkey.to_openssh().unwrap_or_else(|_| "error".to_string());
                    debug!("Public key {} (OpenSSH format): {}", index, auth_key_format);

                    identities.push(Identity {
                        pubkey: key_data.clone(),
                        comment: self.get_cached_key_name(index),
                    });
                }
                Err(e) => {
                    // Log warning but continue with other keys
                    let secret_id = self
                        .secret_ids
                        .get(index)
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    warn!(
                        "Failed to load key at position {} (secret ID: {}): {}. Skipping this key.",
                        index, secret_id, e
                    );
                }
            }
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
            match self.get_private_key(index).await {
                Ok(key) => {
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
                Err(e) => {
                    // Log warning and continue trying other keys
                    let secret_id = self
                        .secret_ids
                        .get(index)
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    warn!(
                        "Failed to load key {} (secret ID: {}) while signing: {}. Trying next key.",
                        index, secret_id, e
                    );
                    continue;
                }
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

#[cfg(test)]
mod tests {
    use ssh_agent_lib::proto::Unparsed;

    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // load key from file
    fn load_key_from_file(path: &str) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to load key from file: {}", e))
    }

    fn get_test_ed25519_key() -> String {
        load_key_from_file("test-data/id_ed25519_testkey")
    }
    fn get_test_rsa_key() -> String {
        load_key_from_file("test-data/id_rsa_testkey")
    }

    // Mock SecretFetcher for testing
    #[derive(Clone)]
    struct MockSecretFetcher {
        secrets: Arc<Mutex<HashMap<Uuid, SecretData>>>,
        call_count: Arc<AtomicUsize>,
        fail_on: Arc<Mutex<Vec<Uuid>>>, // IDs that should fail
    }

    impl MockSecretFetcher {
        fn new() -> Self {
            Self {
                secrets: Arc::new(Mutex::new(HashMap::new())),
                call_count: Arc::new(AtomicUsize::new(0)),
                fail_on: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_secret(&self, id: Uuid, name: String, value: String) {
            let mut secrets = self.secrets.lock().unwrap();
            secrets.insert(id, SecretData { name, value });
        }

        fn set_fail_on(&self, ids: Vec<Uuid>) {
            let mut fail_on = self.fail_on.lock().unwrap();
            *fail_on = ids;
        }

        fn get_call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }

        fn reset_call_count(&self) {
            self.call_count.store(0, Ordering::SeqCst);
        }
    }

    #[async_trait]
    impl SecretFetcher for MockSecretFetcher {
        async fn get_secret(&self, id: Uuid) -> Result<SecretData> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            // Check if this ID should fail
            let fail_on = self.fail_on.lock().unwrap();
            if fail_on.contains(&id) {
                return Err(anyhow::anyhow!("Mock error: Secret not found"));
            }

            let secrets = self.secrets.lock().unwrap();
            secrets
                .get(&id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Secret not found in mock"))
        }
    }

    #[tokio::test]
    async fn test_request_identities_returns_public_keys() {
        // Arrange: Setup mock with a test key
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(
            secret_id,
            "test-key-ed25519".to_string(),
            get_test_ed25519_key().to_string(),
        );

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id]);

        // Act: Request identities
        let identities = agent.request_identities().await.unwrap();

        // Assert: Should return one identity
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].comment, "test-key-ed25519");

        // Verify the key was fetched
        assert_eq!(mock.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_request_identities_with_multiple_keys() {
        // Arrange: Setup mock with two keys
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id1 = Uuid::new_v4();
        let secret_id2 = Uuid::new_v4();

        mock.add_secret(secret_id1, "key-1".to_string(), get_test_ed25519_key());
        mock.add_secret(secret_id2, "key-2".to_string(), get_test_ed25519_key());

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id1, secret_id2]);

        // Act
        let identities = agent.request_identities().await.unwrap();

        // Assert: Should return two identities
        assert_eq!(identities.len(), 2);
        assert_eq!(identities[0].comment, "key-1");
        assert_eq!(identities[1].comment, "key-2");
    }

    #[tokio::test]
    async fn test_request_identities_continues_on_partial_failure() {
        // Arrange: Setup with two keys, one fails
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id1 = Uuid::new_v4();
        let secret_id2 = Uuid::new_v4();

        mock.add_secret(secret_id1, "good-key".to_string(), get_test_ed25519_key());
        // secret_id2 will fail because it's not added
        mock.set_fail_on(vec![secret_id2]);

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id1, secret_id2]);

        // Act
        let identities = agent.request_identities().await.unwrap();

        // Assert: Should return only the successful key
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].comment, "good-key");
    }

    #[tokio::test]
    async fn test_caching_behavior() {
        // Arrange
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(secret_id, "cached-key".to_string(), get_test_ed25519_key());

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id]);

        // Act: Request identities twice
        let _ = agent.request_identities().await.unwrap();
        mock.reset_call_count();
        let _ = agent.request_identities().await.unwrap();

        // Assert: Second request should use cache (0 additional fetches)
        assert_eq!(mock.get_call_count(), 0);
    }

    #[tokio::test]
    async fn test_sign_with_valid_key() {
        // Arrange
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(secret_id, "signing-key".to_string(), get_test_ed25519_key());

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id]);

        // First, get the public key to use in sign request
        let identities = agent.request_identities().await.unwrap();
        let pubkey = identities[0].pubkey.clone();

        // Create a sign request
        let sign_request = SignRequest {
            pubkey,
            data: b"test data to sign".to_vec(),
            flags: 0,
        };

        // Act: Sign the data
        let result = agent.sign(sign_request).await;

        // Assert: Should succeed
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert!(!signature.as_bytes().is_empty());
    }

    #[tokio::test]
    async fn test_sign_with_unknown_key() {
        // Arrange: Agent with one key
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(secret_id, "known-key".to_string(), get_test_ed25519_key());

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id]);

        // Load the agent's key
        let _ = agent.request_identities().await;

        // Create a different key (not in agent) and try to sign with it
        let different_key = PrivateKey::from_openssh(get_test_rsa_key()).unwrap();
        let different_pubkey = different_key.public_key();

        let sign_request = SignRequest {
            pubkey: different_pubkey.key_data().clone(),
            data: b"test data".to_vec(),
            flags: 0,
        };

        // Act: Try to sign with a key not in the agent
        let result = agent.sign(sign_request).await;

        // Assert: Should fail with "Key not found"
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cached_key_name_placeholder() {
        // Arrange: Agent with no loaded keys
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(secret_id, "".to_string(), get_test_ed25519_key());

        let agent = BitwardenAgent::new(mock, vec![secret_id]);

        // Act: Get key name before loading
        let name = agent.get_cached_key_name(0);

        // Assert: Should return placeholder
        assert_eq!(name, "bitwarden-sdk-key");
    }

    #[tokio::test]
    async fn test_sign_uses_cache() {
        // Arrange
        let mock = Arc::new(MockSecretFetcher::new());
        let secret_id = Uuid::new_v4();
        mock.add_secret(
            secret_id,
            "cached-signing-key".to_string(),
            get_test_ed25519_key(),
        );

        let mut agent = BitwardenAgent::new(mock.clone(), vec![secret_id]);

        // Load key into cache
        let identities = agent.request_identities().await.unwrap();
        let pubkey = identities[0].pubkey.clone();

        // Reset call count
        mock.reset_call_count();

        // Act: Sign (should use cached key)
        let sign_request = SignRequest {
            pubkey,
            data: b"test data".to_vec(),
            flags: 0,
        };
        let _ = agent.sign(sign_request).await.unwrap();

        // Assert: Should not fetch again (uses cache)
        assert_eq!(mock.get_call_count(), 0);
    }
}
