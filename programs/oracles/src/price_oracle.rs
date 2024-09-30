use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

// Define constants
const MAX_SWITCHBOARD_DATA_AGE: i64 = 300; // 5 minutes
const SWITCHBOARD_CONFIDENCE_INTERVAL: f64 = 0.80;

/// Represents the different types of assets supported by the oracle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssetType {
    JupSOL,
    MSOL,
    VSOL,
    BSOL,
    HSOL,
    JitoSOL,
    SOL,
}

/// Wrapper for AssetType to implement Default trait
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum AssetTypeWrapper {
    #[default]
    JupSOL,
    MSOL,
    VSOL,
    BSOL,
    HSOL,
    JitoSOL,
    SOL,
}

impl From<AssetType> for AssetTypeWrapper {
    fn from(asset_type: AssetType) -> Self {
        match asset_type {
            AssetType::JupSOL => AssetTypeWrapper::JupSOL,
            AssetType::MSOL => AssetTypeWrapper::MSOL,
            AssetType::VSOL => AssetTypeWrapper::VSOL,
            AssetType::BSOL => AssetTypeWrapper::BSOL,
            AssetType::HSOL => AssetTypeWrapper::HSOL,
            AssetType::JitoSOL => AssetTypeWrapper::JitoSOL,
            AssetType::SOL => AssetTypeWrapper::SOL,
        }
    }
}

impl From<AssetTypeWrapper> for AssetType {
    fn from(wrapper: AssetTypeWrapper) -> Self {
        match wrapper {
            AssetTypeWrapper::JupSOL => AssetType::JupSOL,
            AssetTypeWrapper::MSOL => AssetType::MSOL,
            AssetTypeWrapper::VSOL => AssetType::VSOL,
            AssetTypeWrapper::BSOL => AssetType::BSOL,
            AssetTypeWrapper::HSOL => AssetType::HSOL,
            AssetTypeWrapper::JitoSOL => AssetType::JitoSOL,
            AssetTypeWrapper::SOL => AssetType::SOL,
        }
    }
}

/// Represents the price data for an asset
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
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
    pub asset_count: u8,
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
    pub price_data: Vec<PriceData>,
    pub asset_types: Vec<AssetTypeWrapper>,
    pub bump: u8,
}

/// Main struct for the Price Oracle
pub struct PriceOracle;

impl PriceOracle {
    pub const MAX_ASSETS: usize = 10;
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
        header.asset_count = 0;
        header.last_global_update = 0;
        header.emergency_stop = false;
        header.authority = authority.key();
        header.switchboard_program_id = switchboard_program_id;
        header.bump = header_bump;

        data.price_data = Vec::new();
        data.asset_types = Vec::new();
        data.bump = data_bump;

        Ok(())
    }

    /// Updates the price and APY for a specific asset
    pub fn update_price_and_apy(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        feed: &AccountLoader<AggregatorAccountData>,
        asset_type: AssetType,
        clock: &Clock
    ) -> Result<()> {
        if header.emergency_stop {
            msg!("Emergency stop is activated. Update aborted.");
            return Err(error!(OracleError::EmergencyStop));
        }

        let (new_price, new_apy) = Self::get_price_and_apy_from_feed(feed, &header.switchboard_program_id, asset_type)?;
        let current_time = clock.unix_timestamp;

        let index = Self::find_or_add_asset(header, data, asset_type)?;
        if let Some(price_data) = data.price_data.get_mut(index) {
            let price_change = (new_price - price_data.price).abs() / price_data.price;
            if price_change > 0.2 {
                msg!("Price change exceeds 20% limit. Old price: {}, New price: {}", price_data.price, new_price);
                header.emergency_stop = true;
                return Err(error!(OracleError::PriceChangeExceedsLimit));
            }

            price_data.last_price = price_data.price;
            price_data.price = new_price;
            price_data.apy = new_apy;
            price_data.last_update_time = current_time;
            msg!("Price and APY updated for {:?}. New price: {}, New APY: {}", asset_type, new_price, new_apy);
        } else {
            msg!("Invalid index for asset type {:?}", asset_type);
            return Err(error!(OracleError::InvalidIndex));
        }

        header.last_global_update = current_time;
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

        let new_price = Self::get_sol_price_from_feed(feed, &header.switchboard_program_id)?;
        let current_time = clock.unix_timestamp;

        let index = Self::find_or_add_asset(header, data, AssetType::SOL)?;
        if let Some(price_data) = data.price_data.get_mut(index) {
            let price_change = (new_price - price_data.price).abs() / price_data.price;
            if price_change > 0.2 {
                msg!("SOL price change exceeds 20% limit. Old price: {}, New price: {}", price_data.price, new_price);
                header.emergency_stop = true;
                return Err(error!(OracleError::PriceChangeExceedsLimit));
            }

            price_data.last_price = price_data.price;
            price_data.price = new_price;
            price_data.last_update_time = current_time;
            msg!("SOL price updated. New price: {}", new_price);
        } else {
            msg!("Invalid index for SOL");
            return Err(error!(OracleError::InvalidIndex));
        }

        header.last_global_update = current_time;
        Ok(())
    }

    /// Gets the current price for a specific asset
    pub fn get_current_price(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = Self::find_asset(data, asset_type)?;
        data.price_data.get(index)
            .map(|price_data| price_data.price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    /// Gets the current APY for a specific asset
    pub fn get_current_apy(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = Self::find_asset(data, asset_type)?;
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
    }

    /// Finds the index of an asset in the data account
    fn find_asset(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<usize> {
        let wrapper: AssetTypeWrapper = asset_type.into();
        data.asset_types.iter()
            .position(|&at| at == wrapper)
            .ok_or_else(|| error!(OracleError::AssetNotFound))
    }

    /// Finds or adds an asset to the data account
    fn find_or_add_asset(header: &mut Account<PriceOracleHeader>, data: &mut Account<PriceOracleData>, asset_type: AssetType) -> Result<usize> {
        if let Ok(index) = Self::find_asset(data, asset_type) {
            Ok(index)
        } else if (header.asset_count as usize) < Self::MAX_ASSETS {
            let index = header.asset_count as usize;
            data.asset_types.push(asset_type.into());
            data.price_data.push(PriceData::default());
            header.asset_count += 1;
            Ok(index)
        } else {
            Err(error!(OracleError::MaxAssetsReached))
        }
    }

    /// Gets the price and APY from the Switchboard feed
    fn get_price_and_apy_from_feed(feed: &AccountLoader<AggregatorAccountData>, switchboard_program_id: &Pubkey, asset_type: AssetType) -> Result<(f64, f64)> {
        let feed_data = feed.load().map_err(|_| error!(OracleError::InvalidSwitchboardAccount))?;

        if feed.to_account_info().owner != switchboard_program_id {
            msg!("Invalid Switchboard account owner");
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let result = feed_data.get_result()?;
        let result_f64 = Self::switchboard_decimal_to_f64(&result);
        let values: Vec<f64> = result_f64.to_string().split(',').filter_map(|s| s.parse().ok()).collect();

        if values.len() < 12 {
            return Err(error!(OracleError::InvalidSwitchboardData));
        }

        let (price, apy) = match asset_type {
            AssetType::JupSOL => (values[0], values[1]),
            AssetType::VSOL => (values[2], values[3]),
            AssetType::BSOL => (values[4], values[5]),
            AssetType::MSOL => (values[6], values[7]),
            AssetType::HSOL => (values[8], values[9]),
            AssetType::JitoSOL => (values[10], values[11]),
            AssetType::SOL => return Err(error!(OracleError::InvalidAssetType)),
        };

        let current_timestamp = clock::Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?.unix_timestamp;
        feed_data.check_staleness(current_timestamp, MAX_SWITCHBOARD_DATA_AGE)
            .map_err(|_| {
                msg!("Switchboard data is stale");
                error!(OracleError::StaleData)
            })?;

        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(SWITCHBOARD_CONFIDENCE_INTERVAL))
            .map_err(|_| {
                msg!("Switchboard data exceeds confidence interval");
                error!(OracleError::ExceedsConfidenceInterval)
            })?;

        Ok((price, apy))
    }

    /// Gets the SOL price from the Switchboard feed
    fn get_sol_price_from_feed(feed: &AccountLoader<AggregatorAccountData>, switchboard_program_id: &Pubkey) -> Result<f64> {
        let feed_data = feed.load().map_err(|_| error!(OracleError::InvalidSwitchboardAccount))?;

        if feed.to_account_info().owner != switchboard_program_id {
            msg!("Invalid Switchboard account owner");
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let result = feed_data.get_result()?;
        let price = Self::switchboard_decimal_to_f64(&result);

        let current_timestamp = clock::Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?.unix_timestamp;
        feed_data.check_staleness(current_timestamp, MAX_SWITCHBOARD_DATA_AGE)
            .map_err(|_| {
                msg!("Switchboard data is stale");
                error!(OracleError::StaleData)
            })?;

        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(SWITCHBOARD_CONFIDENCE_INTERVAL))
            .map_err(|_| {
                msg!("Switchboard data exceeds confidence interval");
                error!(OracleError::ExceedsConfidenceInterval)
            })?;

        Ok(price)
    }

    /// Converts a SwitchboardDecimal to f64
    fn switchboard_decimal_to_f64(decimal: &SwitchboardDecimal) -> f64 {
        let mantissa = decimal.mantissa;
        let scale = decimal.scale;
        (mantissa as f64) * 10f64.powi(-(scale as i32))
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
    #[msg("Invalid price feed")]
    InvalidPriceFeed,
    #[msg("Invalid APY feed")]
    InvalidApyFeed,
    #[msg("Price not available")]
    PriceNotAvailable,
    #[msg("APY not available")]
    ApyNotAvailable,
    #[msg("Invalid decimal conversion")]
    InvalidDecimalConversion,
    #[msg("Price change exceeds 20% limit")]
    PriceChangeExceedsLimit,
    #[msg("Emergency stop activated")]
    EmergencyStop,
    #[msg("Invalid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Stale data")]
    StaleData,
    #[msg("Exceeds confidence interval")]
    ExceedsConfidenceInterval,
    #[msg("Maximum number of assets reached")]
    MaxAssetsReached,
    #[msg("Asset not found")]
    AssetNotFound,
    #[msg("Invalid index")]
    InvalidIndex,
    #[msg("Clock unavailable")]
    ClockUnavailable,
    #[msg("Invalid Switchboard program")]
    InvalidSwitchboardProgram,
    #[msg("Invalid Switchboard data")]
    InvalidSwitchboardData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switchboard_decimal_to_f64() {
        let decimal = SwitchboardDecimal {
            mantissa: 1234567890,
            scale: 9,
        };
        let result = PriceOracle::switchboard_decimal_to_f64(&decimal);
        assert_eq!(result, 1.23456789);
    }

    // Add more tests here as needed
}