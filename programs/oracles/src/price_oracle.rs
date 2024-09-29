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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq)]
pub struct AssetTypeWrapper(u8);

impl From<AssetType> for AssetTypeWrapper {
    fn from(asset_type: AssetType) -> Self {
        AssetTypeWrapper(asset_type as u8)
    }
}

impl From<AssetTypeWrapper> for AssetType {
    fn from(wrapper: AssetTypeWrapper) -> Self {
        match wrapper.0 {
            0 => AssetType::JupSOL,
            1 => AssetType::MSOL,
            2 => AssetType::BSOL,
            3 => AssetType::HSOL,
            4 => AssetType::JitoSOL,
            5 => AssetType::VSOL,
            6 => AssetType::SOL,
            _ => panic!("Invalid AssetType"),
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
pub struct PriceOracleHeader {
    pub asset_count: u8,
    pub last_global_update: i64,
    pub emergency_stop: bool,
    pub authority: Pubkey,
}

#[account]
pub struct PriceOracleData {
    pub price_data: Vec<PriceData>,
    pub asset_types: Vec<AssetTypeWrapper>,
}

pub struct PriceOracle<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceOracleData>,
}

impl<'info> PriceOracle<'info> {
    pub const MAX_ASSETS: usize = 10;

    pub fn initialize(header: &mut Account<PriceOracleHeader>, data: &mut Account<PriceOracleData>, authority: &Signer) -> Result<()> {
        header.asset_count = 0;
        header.last_global_update = 0;
        header.emergency_stop = false;
        header.authority = authority.key();

        data.price_data = Vec::new();
        data.asset_types = Vec::new();

        Ok(())
    }

    pub fn update_price(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()> {
        if self.header.emergency_stop {
            return Err(error!(OracleError::EmergencyStop));
        }

        let new_price = Self::get_price_from_feed(feed)?;
        let current_time = clock.unix_timestamp;

        let index = self.find_or_add_asset(asset_type)?;
        if let Some(price_data) = self.data.price_data.get_mut(index) {
            let price_change = (new_price - price_data.price).abs() / price_data.price;
            if price_change > 0.2 {
                self.header.emergency_stop = true;
                return Err(error!(OracleError::PriceChangeExceedsLimit));
            }

            price_data.last_price = price_data.price;
            price_data.price = new_price;
            price_data.last_update_time = current_time;
        } else {
            return Err(error!(OracleError::InvalidIndex));
        }

        self.header.last_global_update = current_time;
        Ok(())
    }

    pub fn update_apy(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()> {
        if self.header.emergency_stop {
            return Err(error!(OracleError::EmergencyStop));
        }

        let apy = Self::get_apy_from_feed(feed)?;
        let current_time = clock.unix_timestamp;

        let index = self.find_or_add_asset(asset_type)?;
        if let Some(price_data) = self.data.price_data.get_mut(index) {
            price_data.apy = apy;
            price_data.last_update_time = current_time;
        } else {
            return Err(error!(OracleError::InvalidIndex));
        }

        self.header.last_global_update = current_time;
        Ok(())
    }

    pub fn get_current_price(&self, asset_type: AssetType) -> Result<f64> {
        let index = self.find_asset(asset_type)?;
        self.data.price_data.get(index)
            .map(|price_data| price_data.price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn get_last_price(&self, asset_type: AssetType) -> Result<f64> {
        let index = self.find_asset(asset_type)?;
        self.data.price_data.get(index)
            .map(|price_data| price_data.last_price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn get_current_apy(&self, asset_type: AssetType) -> Result<f64> {
        let index = self.find_asset(asset_type)?;
        self.data.price_data.get(index)
            .map(|price_data| price_data.apy)
            .ok_or_else(|| error!(OracleError::ApyNotAvailable))
    }

    pub fn last_update_time(&self, asset_type: AssetType) -> Result<i64> {
        let index = self.find_asset(asset_type)?;
        self.data.price_data.get(index)
            .map(|price_data| price_data.last_update_time)
            .ok_or_else(|| error!(OracleError::DataNotAvailable))
    }

    pub fn is_emergency_stopped(&self) -> bool {
        self.header.emergency_stop
    }

    pub fn set_emergency_stop(&mut self, stop: bool) {
        self.header.emergency_stop = stop;
    }

    fn find_asset(&self, asset_type: AssetType) -> Result<usize> {
        let wrapper: AssetTypeWrapper = asset_type.into();
        self.data.asset_types.iter()
            .position(|&at| at == wrapper)
            .ok_or_else(|| error!(OracleError::AssetNotFound))
    }

    fn find_or_add_asset(&mut self, asset_type: AssetType) -> Result<usize> {
        if let Ok(index) = self.find_asset(asset_type) {
            Ok(index)
        } else if (self.header.asset_count as usize) < Self::MAX_ASSETS {
            let index = self.header.asset_count as usize;
            self.data.asset_types.push(asset_type.into());
            self.data.price_data.push(PriceData::default());
            self.header.asset_count += 1;
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