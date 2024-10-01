use anchor_lang::prelude::*;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};
use std::convert::TryInto;
use crate::price_oracle::OracleError;

pub const DEVNET_AGGREGATOR_PUBKEY: &str = "4NiWaTuje7SVe9DN1vfnX7m1qBC7DnUxwRxbdgEDUGX1";
pub const SOL_PRICE_AGGREGATOR_PUBKEY: &str = "GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR";
pub const DEFAULT_DEVNET_QUEUE: &str = "EYiAmGSdsQTuCw413V5BzaruWuCCSDgTPtBGvLkXHbe7";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwitchboardResult {
    pub value: f64,
}

impl SwitchboardResult {
    pub fn new(value: f64) -> Self {
        SwitchboardResult { value }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MultiAssetResult {
    pub prices: [f64; 6],
    pub apys: [f64; 6],
}

pub fn get_switchboard_result(
    switchboard_feed: &AccountLoader<AggregatorAccountData>,
) -> Result<SwitchboardResult> {
    let feed = switchboard_feed.load().map_err(|e| {
        msg!("Failed to load Switchboard feed: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    let result = feed.get_result().map_err(|e| {
        msg!("Failed to get result from Switchboard feed: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    switchboard_decimal_to_result(&result).map_err(|e| {
        msg!("Failed to convert Switchboard result: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })
}

pub fn get_multi_asset_result(
    switchboard_feed: &AccountLoader<AggregatorAccountData>,
) -> Result<MultiAssetResult> {
    let feed = switchboard_feed.load().map_err(|e| {
        msg!("Failed to load Switchboard feed: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    let result = feed.get_result().map_err(|e| {
        msg!("Failed to get result from Switchboard feed: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    parse_multi_asset_data(&result).map_err(|e| {
        msg!("Failed to parse multi-asset data: {:?}", e);
        Error::from(OracleError::InvalidSwitchboardData)
    })
}

pub fn get_sol_price(
    switchboard_feed: &AccountLoader<AggregatorAccountData>,
) -> Result<SwitchboardResult> {
    let feed = switchboard_feed.load().map_err(|e| {
        msg!("Failed to load Switchboard feed for SOL price: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    let result = feed.get_result().map_err(|e| {
        msg!("Failed to get result from Switchboard feed for SOL price: {:?}", e);
        Error::from(OracleError::InvalidAccountData)
    })?;

    parse_sol_price(&result).map_err(|e| {
        msg!("Failed to parse SOL price: {:?}", e);
        Error::from(OracleError::InvalidSwitchboardData)
    })
}

fn switchboard_decimal_to_result(decimal: &SwitchboardDecimal) -> std::result::Result<SwitchboardResult, OracleError> {
    let mantissa = decimal.mantissa;
    let scale = decimal.scale;

    let value = (mantissa as f64) * 10f64.powi(-(scale as i32));
    
    if value.is_finite() {
        msg!("Switchboard result converted successfully: {}", value);
        Ok(SwitchboardResult { value })
    } else {
        msg!("Switchboard result is not a finite number: mantissa={}, scale={}", mantissa, scale);
        Err(OracleError::InvalidSwitchboardData)
    }
}

fn parse_sol_price(decimal: &SwitchboardDecimal) -> std::result::Result<SwitchboardResult, OracleError> {
    let result_str = switchboard_decimal_to_string(decimal)?;
    
    // Parse the JSON string
    let json: serde_json::Value = serde_json::from_str(&result_str)
        .map_err(|_| OracleError::InvalidSwitchboardData)?;
    
    // Extract the "result" field
    let result = json["result"].as_str()
        .ok_or(OracleError::InvalidSwitchboardData)?;
    
    // Parse the result as f64
    let value = result.parse::<f64>()
        .map_err(|_| OracleError::InvalidSwitchboardData)?;
    
    Ok(SwitchboardResult { value })
}

fn parse_multi_asset_data(decimal: &SwitchboardDecimal) -> std::result::Result<MultiAssetResult, OracleError> {
    let result_str = switchboard_decimal_to_string(decimal)?;
    let values: Vec<f64> = result_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if values.len() != 12 {
        return Err(OracleError::InvalidSwitchboardData);
    }

    let prices = [
        values[0], values[2], values[4], values[6], values[8], values[10]
    ];
    let apys = [
        values[1], values[3], values[5], values[7], values[9], values[11]
    ];

    Ok(MultiAssetResult { prices, apys })
}

fn switchboard_decimal_to_string(decimal: &SwitchboardDecimal) -> std::result::Result<String, OracleError> {
    let mantissa = decimal.mantissa;
    let scale = decimal.scale;

    let value = (mantissa as f64) * 10f64.powi(-(scale as i32));
    
    if value.is_finite() {
        Ok(value.to_string())
    } else {
        Err(OracleError::InvalidSwitchboardData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switchboard_decimal_to_result() {
        let decimal = SwitchboardDecimal {
            mantissa: 12340000,
            scale: 5,
        };
        let result = switchboard_decimal_to_result(&decimal).unwrap();
        assert_eq!(result.value, 123.4);

        let invalid_decimal = SwitchboardDecimal {
            mantissa: i128::MAX,
            scale: u32::MAX,
        };
        assert!(switchboard_decimal_to_result(&invalid_decimal).is_err());
    }

    #[test]
    fn test_parse_multi_asset_data() {
        let decimal = SwitchboardDecimal {
            mantissa: 8114553583522887934,
            scale: 17,
        };
        let result = parse_multi_asset_data(&decimal).unwrap();
        assert_eq!(result.prices.len(), 6);
        assert_eq!(result.apys.len(), 6);
    }

    #[test]
    fn test_parse_sol_price() {
        let decimal = SwitchboardDecimal {
            mantissa: 15610523850000000000000000000,
            scale: 26,
        };
        let result = parse_sol_price(&decimal).unwrap();
        assert_eq!(result.value, 156.10523850000000000000000000);
    }
}