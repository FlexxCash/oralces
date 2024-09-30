use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

// Define constants
const MAX_SWITCHBOARD_DATA_AGE: i64 = 300; // 5 minutes
const SWITCHBOARD_CONFIDENCE_INTERVAL: f64 = 0.80;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssetType {
    JupSOL,
    MSOL,
    BSOL,
    HSOL,
    JitoSOL,
    VSOL,
    SOL,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum AssetTypeWrapper {
    #[default]
    JupSOL,
    MSOL,
    BSOL,
    HSOL,
    JitoSOL,
    VSOL,
    SOL,
}

impl From<AssetType> for AssetTypeWrapper {
    fn from(asset_type: AssetType) -> Self {
        match asset_type {
            AssetType::JupSOL => AssetTypeWrapper::JupSOL,
            AssetType::MSOL => AssetTypeWrapper::MSOL,
            AssetType::BSOL => AssetTypeWrapper::BSOL,
            AssetType::HSOL => AssetTypeWrapper::HSOL,
            AssetType::JitoSOL => AssetTypeWrapper::JitoSOL,
            AssetType::VSOL => AssetTypeWrapper::VSOL,
            AssetType::SOL => AssetTypeWrapper::SOL,
        }
    }
}

impl From<AssetTypeWrapper> for AssetType {
    fn from(wrapper: AssetTypeWrapper) -> Self {
        match wrapper {
            AssetTypeWrapper::JupSOL => AssetType::JupSOL,
            AssetTypeWrapper::MSOL => AssetType::MSOL,
            AssetTypeWrapper::BSOL => AssetType::BSOL,
            AssetTypeWrapper::HSOL => AssetType::HSOL,
            AssetTypeWrapper::JitoSOL => AssetType::JitoSOL,
            AssetTypeWrapper::VSOL => AssetType::VSOL,
            AssetTypeWrapper::SOL => AssetType::SOL,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PriceData {
    pub price: f64,
    pub last_price: f64,
    pub last_update_time: i64,
    pub apy: f64,
}

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

#[account]
#[derive(Default)]
pub struct PriceOracleData {
    pub price_data: Vec<PriceData>,
    pub asset_types: Vec<AssetTypeWrapper>,
    pub bump: u8,
}

pub struct PriceOracle;

impl PriceOracle {
    pub const MAX_ASSETS: usize = 10;
    pub const HEADER_SEED: &'static [u8] = b"price_oracle_header";
    pub const DATA_SEED: &'static [u8] = b"price_oracle_data";

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

    pub fn update_price(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        feed: &AccountLoader<AggregatorAccountData>,
        asset_type: AssetType,
        clock: &Clock
    ) -> Result<()> {
        if header.emergency_stop {
            msg!("Emergency stop is activated. Price update aborted.");
            return Err(error!(OracleError::EmergencyStop));
        }

        let new_price = Self::get_price_from_feed(feed, &header.switchboard_program_id)?;
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
            price_data.last_update_time = current_time;
            msg!("Price updated for {:?}. New price: {}", asset_type, new_price);
        } else {
            msg!("Invalid index for asset type {:?}", asset_type);
            return Err(error!(OracleError::InvalidIndex));
        }

        header.last_global_update = current_time;
        Ok(())
    }

    pub fn update_apy(
        header: &mut Account<PriceOracleHeader>,
        data: &mut Account<PriceOracleData>,
        feed: &AccountLoader<AggregatorAccountData>,
        asset_type: AssetType,
        clock: &Clock
    ) -> Result<()> {
        if header.emergency_stop {
            return Err(error!(OracleError::EmergencyStop));
        }

        let apy = Self::get_apy_from_feed(feed, &header.switchboard_program_id)?;
        let current_time = clock.unix_timestamp;

        let index = Self::find_or_add_asset(header, data, asset_type)?;
        if let Some(price_data) = data.price_data.get_mut(index) {
            price_data.apy = apy;
            price_data.last_update_time = current_time;
        } else {
            return Err(error!(OracleError::InvalidIndex));
        }

        header.last_global_update = current_time;
        Ok(())
    }

    pub fn get_current_price(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = Self::find_asset(data, asset_type)?;
        data.price_data.get(index)
            .map(|price_data| price_data.price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn get_current_apy(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<f64> {
        let index = Self::find_asset(data, asset_type)?;
        data.price_data.get(index)
            .map(|price_data| price_data.apy)
            .ok_or_else(|| error!(OracleError::ApyNotAvailable))
    }

    pub fn is_emergency_stopped(header: &Account<PriceOracleHeader>) -> bool {
        header.emergency_stop
    }

    pub fn set_emergency_stop(header: &mut Account<PriceOracleHeader>, stop: bool) {
        header.emergency_stop = stop;
    }

    fn find_asset(data: &Account<PriceOracleData>, asset_type: AssetType) -> Result<usize> {
        let wrapper: AssetTypeWrapper = asset_type.into();
        data.asset_types.iter()
            .position(|&at| at == wrapper)
            .ok_or_else(|| error!(OracleError::AssetNotFound))
    }

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

    fn get_price_from_feed(feed: &AccountLoader<AggregatorAccountData>, switchboard_program_id: &Pubkey) -> Result<f64> {
        let feed_data = feed.load().map_err(|_| error!(OracleError::InvalidSwitchboardAccount))?;

        if feed.to_account_info().owner != switchboard_program_id {
            msg!("Invalid Switchboard account owner");
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let switchboard_decimal = feed_data.get_result()?;
        let decimal = convert_switchboard_decimal(switchboard_decimal)?;

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

        Ok(decimal)
    }

    fn get_apy_from_feed(feed: &AccountLoader<AggregatorAccountData>, switchboard_program_id: &Pubkey) -> Result<f64> {
        let feed_data = feed.load().map_err(|_| error!(OracleError::InvalidSwitchboardAccount))?;

        if feed.to_account_info().owner != switchboard_program_id {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let switchboard_decimal = feed_data.get_result()?;
        let decimal = convert_switchboard_decimal(switchboard_decimal)?;

        let current_timestamp = clock::Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?.unix_timestamp;
        feed_data.check_staleness(current_timestamp, MAX_SWITCHBOARD_DATA_AGE)
            .map_err(|_| error!(OracleError::StaleData))?;
        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(SWITCHBOARD_CONFIDENCE_INTERVAL))
            .map_err(|_| error!(OracleError::ExceedsConfidenceInterval))?;

        Ok(decimal)
    }

    pub fn get_price_oracle_header_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::HEADER_SEED], program_id)
    }

    pub fn get_price_oracle_data_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::DATA_SEED], program_id)
    }
}

/// Converts a SwitchboardDecimal to an f64.
///
/// This function is a public wrapper around the private switchboard_decimal_to_f64 function.
/// It's used to convert Switchboard's decimal representation to a standard f64 value.
///
/// # Arguments
///
/// * `decimal` - A SwitchboardDecimal to be converted.
///
/// # Returns
///
/// * `Result<f64>` - The converted f64 value if successful, or an error if the conversion fails.
pub fn convert_switchboard_decimal(decimal: SwitchboardDecimal) -> Result<f64> {
    switchboard_decimal_to_f64(decimal)
}

// Helper function to convert SwitchboardDecimal to f64
fn switchboard_decimal_to_f64(decimal: SwitchboardDecimal) -> Result<f64> {
    let value = (decimal.mantissa as f64) * 10f64.powi(decimal.scale.try_into().unwrap());
    if value.is_infinite() || value.is_nan() {
        return Err(error!(OracleError::SwitchboardConversionError));
    }
    Ok(value)
}

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
    #[msg("Error converting Switchboard decimal")]
    SwitchboardConversionError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_switchboard_decimal() {
        // Test normal case
        let normal_decimal = SwitchboardDecimal {
            mantissa: 12345,
            scale: 2,
        };
        assert_eq!(convert_switchboard_decimal(normal_decimal).unwrap(), 123.45);

        // Test very small number
        let small_decimal = SwitchboardDecimal {
            mantissa: 1,
            scale: 10,
        };
        assert_eq!(convert_switchboard_decimal(small_decimal).unwrap(), 1e-10);

        // Test very large number
        let large_decimal = SwitchboardDecimal {
            mantissa: 1234567890123456,
            scale: -5,
        };
        assert_eq!(convert_switchboard_decimal(large_decimal).unwrap(), 1.234567890123456e20);

        // Test zero
        let zero_decimal = SwitchboardDecimal {
            mantissa: 0,
            scale: 0,
        };
        assert_eq!(convert_switchboard_decimal(zero_decimal).unwrap(), 0.0);

        // Test error case (infinity)
        let infinity_decimal = SwitchboardDecimal {
            mantissa: 1,
            scale: 1000,
        };
        assert!(convert_switchboard_decimal(infinity_decimal).is_err());
    }
}