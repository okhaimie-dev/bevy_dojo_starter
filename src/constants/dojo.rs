use starknet::{core::types::Felt, macros::selector};
use std::env;

/// Configuration for Dojo blockchain integration
#[derive(Debug, Clone)]
pub struct DojoConfig {
    pub torii_url: String,
    pub katana_url: String,
    pub world_address: Felt,
    pub action_address: Felt,
    pub use_dev_account: bool,
    pub dev_account_index: u32,
}

impl Default for DojoConfig {
    fn default() -> Self {
        Self {
            torii_url: env::var("TORII_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            katana_url: env::var("KATANA_URL")
                .unwrap_or_else(|_| "http://0.0.0.0:5050".to_string()),
            world_address: env::var("WORLD_ADDRESS")
                .ok()
                .and_then(|addr| Felt::from_hex(&addr).ok())
                .unwrap_or_else(|| {
                    // Real deployed world address from manifest_dev.json
                    Felt::from_hex_unchecked(
                        "0x058565b92f55fb07b53940b4b7eea3df2ac2878210e5c7a4c68201e8c511a546",
                    )
                }),
            action_address: env::var("ACTION_ADDRESS")
                .ok()
                .and_then(|addr| Felt::from_hex(&addr).ok())
                .unwrap_or_else(|| {
                    // Real deployed action address from manifest_dev.json
                    Felt::from_hex_unchecked(
                        "0x049f9b281bb08aea6d745f28cf31dd529348b04a21d9a5ae1ef19197665c02da",
                    )
                }),
            use_dev_account: env::var("USE_DEV_ACCOUNT").unwrap_or_else(|_| "true".to_string())
                == "true",
            dev_account_index: env::var("DEV_ACCOUNT_INDEX")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }
}

// Contract functions
pub const SPAWN_SELECTOR: Felt = selector!("spawn");
pub const MOVE_SELECTOR: Felt = selector!("move");
