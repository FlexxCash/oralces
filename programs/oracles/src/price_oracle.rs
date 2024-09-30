use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, SWITCHBOARD_PROGRAM_ID};

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
        header_bump: u8,
        data_bump: u8,
    ) -> Result<()> {
        header.asset_count = 0;
        header.last_global_update = 0;
        header.emergency_stop = false;
        header.authority = authority.key();
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
            return Err(error!(OracleError::EmergencyStop));
        }

        let new_price = Self::get_price_from_feed(feed)?;
        let current_time = clock.unix_timestamp;

        let index = Self::find_or_add_asset(header, data, asset_type)?;
        if let Some(price_data) = data.price_data.get_mut(index) {
            let price_change = (new_price - price_data.price).abs() / price_data.price;
            if price_change > 0.2 {
                header.emergency_stop = true;
                return Err(error!(OracleError::PriceChangeExceedsLimit));
            }

            price_data.last_price = price_data.price;
            price_data.price = new_price;
            price_data.last_update_time = current_time;
        } else {
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

        let apy = Self::get_apy_from_feed(feed)?;
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

    fn get_price_from_feed(feed: &AccountLoader<AggregatorAccountData>) -> Result<f64> {
        let feed_data = feed.load()?;

        if feed.to_account_info().owner != &SWITCHBOARD_PROGRAM_ID {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let decimal: f64 = feed_data.get_result()?.try_into()?;

        feed_data.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;
        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(0.80))?;

        Ok(decimal)
    }

    fn get_apy_from_feed(feed: &AccountLoader<AggregatorAccountData>) -> Result<f64> {
        let feed_data = feed.load()?;

        if feed.to_account_info().owner != &SWITCHBOARD_PROGRAM_ID {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let decimal: f64 = feed_data.get_result()?.try_into()?;

        feed_data.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;
        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(0.001))?;

        Ok(decimal)
    }

    pub fn get_price_oracle_header_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::HEADER_SEED], program_id)
    }

    pub fn get_price_oracle_data_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::DATA_SEED], program_id)
    }
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
}