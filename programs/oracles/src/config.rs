use anchor_lang::prelude::*;

// Constants for price oracle
pub const MAX_SWITCHBOARD_DATA_AGE: i64 = 300; // 5 minutes
pub const PRICE_CHANGE_LIMIT: f64 = 0.20; // 20%

// Constants for emergency stop
pub const EMERGENCY_STOP_THRESHOLD: u64 = 10000;
pub const XXUSD_PRICE_EMERGENCY_THRESHOLD: f64 = 0.94;

// Constants for asset management
pub const HEDGE_STRATEGY_TRANSFER_LIMIT: f64 = 0.25; // 25%

// Constants for time intervals
pub const PRICE_UPDATE_INTERVAL: i64 = 300; // 5 minutes
pub const NEW_ASSET_ACTIVATION_DELAY: i64 = 7 * 24 * 60 * 60; // 7 days in seconds

// Pubkeys for various accounts (replace with actual pubkeys when available)
pub const XXUSD_MINT: Pubkey = Pubkey::new_from_array([0; 32]); // Replace with actual XXUSD mint pubkey
pub const TREASURY_ACCOUNT: Pubkey = Pubkey::new_from_array([1; 32]); // Replace with actual treasury account pubkey

// Add any other configuration constants here

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(PRICE_CHANGE_LIMIT > 0.0 && PRICE_CHANGE_LIMIT < 1.0);
        assert!(HEDGE_STRATEGY_TRANSFER_LIMIT > 0.0 && HEDGE_STRATEGY_TRANSFER_LIMIT < 1.0);
        assert!(XXUSD_PRICE_EMERGENCY_THRESHOLD > 0.0 && XXUSD_PRICE_EMERGENCY_THRESHOLD < 1.0);
        assert!(NEW_ASSET_ACTIVATION_DELAY > 0);
    }
}