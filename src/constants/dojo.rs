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
                        "0x0393f8a2d0d47e384c3c61eedc08d2873f5d608f8da7ffb013e5d5aa327ac8f2",
                    )
                }),
            action_address: env::var("ACTION_ADDRESS")
                .ok()
                .and_then(|addr| Felt::from_hex(&addr).ok())
                .unwrap_or_else(|| {
                    // Real deployed action address from manifest_dev.json
                    Felt::from_hex_unchecked(
                        "0x0173922b4b70c89732bcb6f3cf6598afb2020001e469b86fc76a4f2a60a139df",
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
