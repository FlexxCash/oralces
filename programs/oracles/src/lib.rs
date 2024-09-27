use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use switchboard_v2::AggregatorAccountData;

pub mod price_oracle;
use price_oracle::{AssetType, PriceOracle};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod flexxcash_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.price_oracle.initialize()?;
        msg!("Price oracle initialized");
        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, asset_type: AssetType) -> Result<()> {
        let clock = Clock::get()?;
        ctx.accounts.price_oracle.update_price(
            &ctx.accounts.oracle_feed,
            asset_type,
            &clock
        )?;

        let new_price = ctx.accounts.price_oracle.get_current_price(asset_type)?;
        msg!("Updated price for {:?}: {}", asset_type, new_price);
        Ok(())
    }

    pub fn update_apy(ctx: Context<UpdateApy>, asset_type: AssetType) -> Result<()> {
        let clock = Clock::get()?;
        ctx.accounts.price_oracle.update_apy(
            &ctx.accounts.oracle_feed,
            asset_type,
            &clock
        )?;
        let apy = ctx.accounts.price_oracle.get_current_apy(asset_type)?;
        msg!("Updated APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn get_current_price(ctx: Context<GetCurrentPrice>, asset_type: AssetType) -> Result<()> {
        let price = ctx.accounts.price_oracle.get_current_price(asset_type)?;
        msg!("Current price for {:?}: {}", asset_type, price);
        Ok(())
    }

    pub fn get_current_apy(ctx: Context<GetCurrentApy>, asset_type: AssetType) -> Result<()> {
        let apy = ctx.accounts.price_oracle.get_current_apy(asset_type)?;
        msg!("Current APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn get_sol_price(ctx: Context<GetSolPrice>) -> Result<()> {
        let sol_price = ctx.accounts.price_oracle.get_current_price(AssetType::SOL)?;
        msg!("Current SOL price: {}", sol_price);
        Ok(())
    }

    pub fn get_usdc_price(_ctx: Context<GetUsdcPrice>) -> Result<()> {
        msg!("Current USDC price: $1.00");
        Ok(())
    }

    pub fn check_emergency_stop(ctx: Context<CheckEmergencyStop>) -> Result<()> {
        let is_stopped = ctx.accounts.price_oracle.is_emergency_stopped();
        msg!("Emergency stop status: {}", is_stopped);
        Ok(())
    }

    pub fn set_emergency_stop(ctx: Context<SetEmergencyStop>, stop: bool) -> Result<()> {
        ctx.accounts.price_oracle.set_emergency_stop(stop);
        msg!("Emergency stop set to: {}", stop);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 1000)]
    pub price_oracle: Account<'info, PriceOracle>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub price_oracle: Account<'info, PriceOracle>,
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
}

#[derive(Accounts)]
pub struct UpdateApy<'info> {
    #[account(mut)]
    pub price_oracle: Account<'info, PriceOracle>,
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
}

#[derive(Accounts)]
pub struct GetCurrentPrice<'info> {
    pub price_oracle: Account<'info, PriceOracle>,
}

#[derive(Accounts)]
pub struct GetCurrentApy<'info> {
    pub price_oracle: Account<'info, PriceOracle>,
}

#[derive(Accounts)]
pub struct GetSolPrice<'info> {
    pub price_oracle: Account<'info, PriceOracle>,
}

#[derive(Accounts)]
pub struct GetUsdcPrice {}

#[derive(Accounts)]
pub struct CheckEmergencyStop<'info> {
    pub price_oracle: Account<'info, PriceOracle>,
}

#[derive(Accounts)]
pub struct SetEmergencyStop<'info> {
    #[account(mut)]
    pub price_oracle: Account<'info, PriceOracle>,
    pub authority: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Price change exceeds limit")]
    PriceChangeExceedsLimit,
    #[msg("Oracle error occurred")]
    OracleError,
    #[msg("Invalid asset type")]
    InvalidAssetType,
    #[msg("Price not available")]
    PriceNotAvailable,
    #[msg("APY not available")]
    ApyNotAvailable,
    #[msg("Emergency stop activated")]
    EmergencyStop,
}