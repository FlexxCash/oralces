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

impl AssetType {
    pub fn get_feed_address(&self) -> Result<Pubkey> {
        match self {
            AssetType::JupSOL => Pubkey::try_from("3zkXukqF4CBSUAq55uAx1CnGrzDKk3cVAesJ4WLpSzgA"),
            AssetType::MSOL => Pubkey::try_from("mU2inw8URG5s97X8xFhY9y2VsLZSPrwqY3eky4DjEQQ"),
            AssetType::BSOL => Pubkey::try_from("2CNMT1r5mWyWTYYD23UoV9ueq2cHpj2qvZKUwrP5LftU"),
            AssetType::HSOL => Pubkey::try_from("4U1ofakLouLjVHXxMXXwNDkz3eUUSMXMqTyeC16Trpdf"),
            AssetType::JitoSOL => Pubkey::try_from("3UF281FMHbuXKsfqGQKQVExexnvMsHMGoGoo917rVf3g"),
            AssetType::VSOL => Pubkey::try_from("2e2WhmSbWvNR94tXzK1caBEX1uCedXns6xVXfqePctJq"),
            AssetType::SOL => Pubkey::try_from("98tVEYkSyG7Di424o98ETFMTxocb5bCWARAeuUF1haL4"),
        }.map_err(|_| error!(OracleError::InvalidAssetType))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PriceData {
    pub price: f64,
    pub last_price: f64,
    pub last_update_time: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ApyData {
    pub apy: f64,
    pub last_update_time: i64,
}

#[account]
pub struct PriceOracle {
    pub prices: Vec<(AssetType, PriceData)>,
    pub apys: Vec<(AssetType, ApyData)>,
    pub last_global_update: i64,
    pub emergency_stop: bool,
}

impl PriceOracle {
    pub fn initialize(&mut self) -> Result<()> {
        self.prices = Vec::new();
        self.apys = Vec::new();
        self.last_global_update = 0;
        self.emergency_stop = false;
        Ok(())
    }

    pub fn update_price(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()> {
        if self.emergency_stop {
            return Err(error!(OracleError::EmergencyStop));
        }

        let new_price = Self::get_price_from_feed(feed)?;
        let current_time = clock.unix_timestamp;

        if let Some(price_data) = self.prices.iter_mut().find(|(at, _)| *at == asset_type) {
            let price_change = (new_price - price_data.1.price).abs() / price_data.1.price;
            if price_change > 0.2 {
                self.emergency_stop = true;
                return Err(error!(OracleError::PriceChangeExceedsLimit));
            }

            price_data.1.last_price = price_data.1.price;
            price_data.1.price = new_price;
            price_data.1.last_update_time = current_time;
        } else {
            self.prices.push((asset_type, PriceData { price: new_price, last_price: new_price, last_update_time: current_time }));
        }

        self.last_global_update = current_time;
        Ok(())
    }

    pub fn update_apy(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()> {
        if self.emergency_stop {
            return Err(error!(OracleError::EmergencyStop));
        }

        let apy = Self::get_apy_from_feed(feed)?;
        let current_time = clock.unix_timestamp;

        if let Some(apy_data) = self.apys.iter_mut().find(|(at, _)| *at == asset_type) {
            apy_data.1.apy = apy;
            apy_data.1.last_update_time = current_time;
        } else {
            self.apys.push((asset_type, ApyData { apy, last_update_time: current_time }));
        }

        self.last_global_update = current_time;
        Ok(())
    }

    pub fn get_current_price(&self, asset_type: AssetType) -> Result<f64> {
        self.prices.iter()
            .find(|(at, _)| *at == asset_type)
            .map(|(_, price_data)| price_data.price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn get_last_price(&self, asset_type: AssetType) -> Result<f64> {
        self.prices.iter()
            .find(|(at, _)| *at == asset_type)
            .map(|(_, price_data)| price_data.last_price)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn get_current_apy(&self, asset_type: AssetType) -> Result<f64> {
        self.apys.iter()
            .find(|(at, _)| *at == asset_type)
            .map(|(_, apy_data)| apy_data.apy)
            .ok_or_else(|| error!(OracleError::ApyNotAvailable))
    }

    pub fn last_update_time(&self, asset_type: AssetType) -> Result<i64> {
        self.prices.iter()
            .find(|(at, _)| *at == asset_type)
            .map(|(_, price_data)| price_data.last_update_time)
            .ok_or_else(|| error!(OracleError::PriceNotAvailable))
    }

    pub fn is_emergency_stopped(&self) -> bool {
        self.emergency_stop
    }

    pub fn set_emergency_stop(&mut self, stop: bool) {
        self.emergency_stop = stop;
    }

    fn get_price_from_feed(feed: &AccountLoader<AggregatorAccountData>) -> Result<f64> {
        let feed_data = feed.load()?;

        // 檢查 feed 所有者
        if feed.to_account_info().owner != &SWITCHBOARD_PROGRAM_ID {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        // 獲取結果
        let decimal: f64 = feed_data.get_result()?.try_into()?;

        // 檢查 feed 是否在過去 5 分鐘內更新
        feed_data.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;

        // 檢查 feed 是否超出 +/- $0.80 的置信區間
        feed_data.check_confidence_interval(SwitchboardDecimal::from_f64(0.80))?;

        Ok(decimal)
    }

    fn get_apy_from_feed(feed: &AccountLoader<AggregatorAccountData>) -> Result<f64> {
        let feed_data = feed.load()?;

        // 檢查 feed 所有者
        if feed.to_account_info().owner != &SWITCHBOARD_PROGRAM_ID {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        // 獲取結果
        let decimal: f64 = feed_data.get_result()?.try_into()?;

        // 檢查 feed 是否在過去 5 分鐘內更新
        feed_data.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;

        // 檢查 feed 是否超出 +/- 0.1% 的置信區間
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
}