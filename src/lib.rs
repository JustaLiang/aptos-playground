use anyhow::{anyhow, Result};
use aptos_sdk::crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use aptos_sdk::move_types::account_address::AccountAddress;
pub use aptos_sdk::rest_client::Client;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

const APTOS_CONFIG_PATH: &str = ".aptos/config.yaml";

/// Config saved to `.aptos/config.yaml`
#[derive(Debug, Serialize, Deserialize)]
pub struct AptosConfig {
    /// Map of profile configs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<BTreeMap<String, ProfileConfig>>,
}

/// An individual profile
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Private key for commands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<Ed25519PrivateKey>,
    /// Public key for commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<Ed25519PublicKey>,
    /// Account for commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountAddress>,
    /// URL for the Aptos rest endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_url: Option<String>,
    /// URL for the Faucet endpoint (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faucet_url: Option<String>,
}

impl AptosConfig {
    pub fn load_profile(profile_key: &str) -> Result<ProfileConfig> {
        let file_content = &String::from_utf8(fs::read(Path::new(APTOS_CONFIG_PATH))?)?;
        let file_content: AptosConfig = serde_yaml::from_str(file_content)?;
        if let Some(mut profiles) = file_content.profiles {
            if let Some(profile) = profiles.remove(profile_key) {
                Ok(profile)
            } else {
                Err(anyhow!("'{}' profile not found", profile_key))
            }
        } else {
            Err(anyhow!("'profiles' feild not found"))
        }
    }
}

#[async_trait]
pub trait ExtentedAptosClient {
    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64>;
}

#[async_trait]
impl ExtentedAptosClient for Client {
    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        let account_resp = self
            .get_account(address)
            .await
            .map_err(|err| anyhow!(err.to_string()))?;
        let account = account_resp.inner();
        Ok(account.sequence_number)
    }
}
