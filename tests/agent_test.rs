#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use ssh_agent_lib::agent::Session;
    use ssh_agent_lib::proto::SignRequest;
    use ssh_key::rand_core::OsRng;
    use ssh_key::{Algorithm, LineEnding, PrivateKey};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;
    use vault_conductor::bitwarden::agent::{BitwardenAgent, SecretFetcher};

    // --- Mocks ---

    #[derive(Clone)]
    struct MockFetcher {
        // The PEM string to return
        secret_value: String,
        // To track how many times we called "Bitwarden"
        call_count: Arc<Mutex<usize>>,
        // Simulate failure?
        should_fail: bool,
    }

    impl MockFetcher {
        fn new(pem: String) -> Self {
            Self {
                secret_value: pem,
                call_count: Arc::new(Mutex::new(0)),
                should_fail: false,
            }
        }

        fn new_failing() -> Self {
            Self {
                secret_value: "".to_string(),
                call_count: Arc::new(Mutex::new(0)),
                should_fail: true,
            }
        }

        fn calls(&self) -> usize {
            *self.call_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl SecretFetcher for MockFetcher {
        async fn get_secret_value(&self, _id: Uuid) -> Result<String> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            if self.should_fail {
                return Err(anyhow!("Network error"));
            }
            Ok(self.secret_value.clone())
        }
    }

    // --- Helper to generate a real SSH key for testing ---
    fn generate_test_key() -> (PrivateKey, String) {
        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let pem = key.to_openssh(LineEnding::LF).unwrap().to_string();
        (key, pem)
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_request_identities_returns_correct_key() {
        let (real_key, pem) = generate_test_key();
        let fetcher = Arc::new(MockFetcher::new(pem));
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        // Act
        let identities = agent
            .request_identities()
            .await
            .expect("Failed to list identities");

        // Assert
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].comment, "bitwarden-sdk-key");
        assert_eq!(
            identities[0].pubkey,
            real_key.public_key().key_data().clone()
        );

        // Verify it called the fetcher
        assert_eq!(fetcher.calls(), 1);
    }

    #[tokio::test]
    async fn test_caching_mechanism() {
        let (_, pem) = generate_test_key();
        let fetcher = Arc::new(MockFetcher::new(pem));
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        // Act: Call twice
        let _ = agent.request_identities().await;
        let _ = agent.request_identities().await;

        // Assert: Fetcher should only be called once due to caching logic in BitwardenAgent
        assert_eq!(
            fetcher.calls(),
            1,
            "Should have cached the key after first call"
        );
    }

    #[tokio::test]
    async fn test_signing_works() {
        let (real_key, pem) = generate_test_key();
        let fetcher = Arc::new(MockFetcher::new(pem));
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        // Data to sign
        let data = b"hello world".to_vec();
        let pubkey = real_key.public_key().key_data().clone();

        let req = SignRequest {
            pubkey,
            data: data.clone(),
            flags: 0,
        };

        // Act
        let signature = agent.sign(req).await.expect("Signing failed");

        // Assert: Verify we got a signature (basic check)
        assert!(
            !signature.as_bytes().is_empty(),
            "Signature should not be empty"
        );
    }

    #[tokio::test]
    async fn test_signing_fails_with_wrong_key_request() {
        let (_key1, pem1) = generate_test_key();
        let (key2, _) = generate_test_key(); // A different key

        let fetcher = Arc::new(MockFetcher::new(pem1));
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        let req = SignRequest {
            pubkey: key2.public_key().key_data().clone(), // Asking for Key2
            data: b"data".to_vec(),
            flags: 0,
        };

        // Act
        let result = agent.sign(req).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key not found"));
    }

    #[tokio::test]
    async fn test_fetch_error_handling() {
        let fetcher = Arc::new(MockFetcher::new_failing());
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        let result = agent.request_identities().await;

        assert!(result.is_err());
        // The error string usually comes formatted from ssh_agent_lib,
        // but the underlying cause should be ours.
        // Check if it bubbles up reasonably.
    }

    #[tokio::test]
    async fn test_extension_returns_none() {
        use ssh_agent_lib::proto::{Extension, Unparsed};

        let (_, pem) = generate_test_key();
        let fetcher = Arc::new(MockFetcher::new(pem));
        let mut agent = BitwardenAgent::new(fetcher.clone(), Uuid::new_v4());

        let ext = Extension {
            name: "test-extension".to_string(),
            details: Unparsed::from(vec![]),
        };

        let result = agent.extension(ext).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
