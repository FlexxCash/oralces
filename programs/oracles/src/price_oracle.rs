use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use std::convert::TryInto;
use switchboard_v2::AggregatorAccountData;
use crate::switchboard_utils::{get_multi_asset_result, get_sol_price, MultiAssetResult, SwitchboardResult};
use crate::config::*;

/// Represents the different types of assets supported by the oracle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssetType {
    JupSOL,
    VSOL,
    BSOL,
    MSOL,
    HSOL,
    JitoSOL,
    SOL,
}

/// Represents the price data for an asset
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PriceData {
    pub price: f64,
    pub last_price: f64,
    pub last_update_time: i64,
    pub apy: f64,
}

/// Represents the header information for the price oracle
#[account]
#[derive(Default)]
pub struct PriceOracleHeader {
    pub last_global_update: i64,
    pub emergency_stop: bool,
    pub authority: Pubkey,
    pub switchboard_program_id: Pubkey,
    pub bump: u8,
}

/// Represents the data storage for the price oracle
#[account]
#[derive(Default)]
pub struct PriceOracleData {
    pub price_data: [PriceData; 7], // 6 assets + SOL
    pub bump: u8,
}

/// Main struct for the Price Oracle
pub struct PriceOracle;

impl PriceOracle {
    pub const HEADER_SEED: &'static [u8] = b"price_oracle_header";
    pub const DATA_SEED: &'static [u8] = b"price_oracle_data";

    /// Initializes the price oracle
    pub fn initialize(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        authority: &Signer,
        switchboard_program_id: Pubkey,
        header_bump: u8,
        data_bump: u8,
    ) -> Result<()> {
        header.last_global_update = 0;
        header.emergency_stop = false;
        header.authority = authority.key();
        header.switchboard_program_id = switchboard_program_id;
        header.bump = header_bump;

        data.price_data = core::array::from_fn(|_| PriceData::default());
        data.bump = data_bump;

        msg!("Price oracle initialized with authority: {}", authority.key());
        Ok(())
    }

    /// Updates the prices and APYs for all assets
    pub fn update_prices_and_apys(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        feed: &AccountLoader<AggregatorAccountData>,
        clock: &Clock
    ) -> Result<()> {
        if header.emergency_stop {
            msg!("Emergency stop is activated. Update aborted.");
            return Err(error!(OracleError::EmergencyStop));
        }

        let multi_asset_result = get_multi_asset_result(feed)?;
        let current_time = clock.unix_timestamp;

        for (i, asset_type) in AssetType::iter().enumerate() {
            if asset_type == AssetType::SOL {
                continue; // SOL is handled separately
            }

            let new_price = multi_asset_result.prices[i];
            let new_apy = multi_asset_result.apys[i];

            Self::update_single_asset_price(data, i, new_price, new_apy, current_time)?;
        }

        header.last_global_update = current_time;
        msg!("All prices and APYs updated successfully at timestamp: {}", current_time);
        Ok(())
    }

    /// Updates the SOL price
    pub fn update_sol_price(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        feed: &AccountLoader<AggregatorAccountData>,
        clock: &Clock
    ) -> Result<()> {
        if header.emergency_stop {
            msg!("Emergency stop is activated. SOL price update aborted.");
            return Err(error!(OracleError::EmergencyStop));
        }

        let sol_price_result = get_sol_price(feed)?;
        let new_price = sol_price_result.value;
        let current_time = clock.unix_timestamp;

        Self::update_single_asset_price(data, 6, new_price, 0.0, current_time)?; // SOL is the last element, APY is not applicable

        header.last_global_update = current_time;
        msg!("SOL price updated successfully. New price: {}", new_price);
        Ok(())
    }

    /// Helper function to update a single asset's price and APY
    fn update_single_asset_price(
        data: &mut Account<PriceOracleData>,
        index: usize,
        new_price: f64,
        new_apy: f64,
        current_time: i64,
    ) -> Result<()> {
        let price_data = &mut data.price_data[index];
        let price_change = (new_price - price_data.price).abs() / price_data.price;
        
        if price_change > PRICE_CHANGE_LIMIT {
            msg!("Price change exceeds {}% limit for asset index {}. Old price: {}, New price: {}", 
                PRICE_CHANGE_LIMIT * 100.0, index, price_data.price, new_price);
            return Err(error!(OracleError::PriceChangeExceedsLimit));
        }

        price_data.last_price = price_data.price;
        price_data.price = new_price;
        price_data.apy = new_apy;
        price_data.last_update_time = current_time;

        msg!("Price and APY updated for asset index {}. New price: {}, New APY: {}", index, new_price, new_apy);
        Ok(())
    }

    /// Gets the current price for a specific asset
    pub fn get_current_price(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = asset_type as usize;
        data.price_data.get(index)
            .map(|price_data| price_data.price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    /// Gets the current APY for a specific asset
    pub fn get_current_apy(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = asset_type as usize;
        data.price_data.get(index)
            .map(|price_data| price_data.apy)
            .ok_or_else(|| error!(OracleError::ApyNotAvailable))
    }

    /// Checks if emergency stop is activated
    pub fn is_emergency_stopped(header: &Account<PriceOracleHeader>) -> bool {
        header.emergency_stop
    }

    /// Sets the emergency stop status
    pub fn set_emergency_stop(header: &mut Account<PriceOracleHeader>, stop: bool) {
        header.emergency_stop = stop;
        msg!("Emergency stop set to: {}", stop);
    }

    /// Gets the PDA for the price oracle header
    pub fn get_price_oracle_header_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::HEADER_SEED], program_id)
    }

    /// Gets the PDA for the price oracle data
    pub fn get_price_oracle_data_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::DATA_SEED], program_id)
    }
}

/// Custom error types for the Oracle
#[error_code]
pub enum OracleError {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
    #[msg("Invalid asset type")]
    InvalidAssetType,
    #[msg("Data not available")]
    DataNotAvailable,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("Price not available")]
    PriceNotAvailable,
    #[msg("APY not available")]
    ApyNotAvailable,
    #[msg("Price change exceeds limit")]
    PriceChangeExceedsLimit,
    #[msg("Emergency stop activated")]
    EmergencyStop,
    #[msg("Invalid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Stale data")]
    StaleData,
    #[msg("Invalid Switchboard data")]
    InvalidSwitchboardData,
}

/// Helper trait to iterate over AssetType
trait AssetTypeIter {
    fn iter() -> impl Iterator<Item = AssetType>;
}

impl AssetTypeIter for AssetType {
    fn iter() -> impl Iterator<Item = AssetType> {
        [
            AssetType::JupSOL,
            AssetType::VSOL,
            AssetType::BSOL,
            AssetType::MSOL,
            AssetType::HSOL,
            AssetType::JitoSOL,
            AssetType::SOL,
        ].into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add tests here as needed
}