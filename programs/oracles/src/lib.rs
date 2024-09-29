use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use switchboard_v2::AggregatorAccountData;

pub mod price_oracle;
use price_oracle::{AssetType, AssetTypeWrapper, PriceOracle, PriceOracleHeader, PriceOracleData};

declare_id!("AtguUUsGDXry7onmb7QqDK4DLwquRkQPsXX1CJTjZsUy");

#[program]
pub mod oracles {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        PriceOracle::initialize(&mut ctx.accounts.header, &mut ctx.accounts.data, &ctx.accounts.authority)?;
        msg!("Price oracle initialized");
        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, asset_type: AssetType) -> Result<()> {
        let clock = Clock::get()?;
        let mut price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        price_oracle.update_price(&ctx.accounts.oracle_feed, asset_type, &clock)?;

        let new_price = price_oracle.get_current_price(asset_type)?;
        msg!("Updated price for {:?}: {}", asset_type, new_price);
        Ok(())
    }

    pub fn update_apy(ctx: Context<UpdateApy>, asset_type: AssetType) -> Result<()> {
        let clock = Clock::get()?;
        let mut price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        price_oracle.update_apy(&ctx.accounts.oracle_feed, asset_type, &clock)?;

        let apy = price_oracle.get_current_apy(asset_type)?;
        msg!("Updated APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn get_current_price(ctx: Context<GetCurrentPrice>, asset_type: AssetType) -> Result<()> {
        let price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        let price = price_oracle.get_current_price(asset_type)?;
        msg!("Current price for {:?}: {}", asset_type, price);
        Ok(())
    }

    pub fn get_current_apy(ctx: Context<GetCurrentApy>, asset_type: AssetType) -> Result<()> {
        let price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        let apy = price_oracle.get_current_apy(asset_type)?;
        msg!("Current APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn get_sol_price(ctx: Context<GetSolPrice>) -> Result<()> {
        let price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        let sol_price = price_oracle.get_current_price(AssetType::SOL)?;
        msg!("Current SOL price: {}", sol_price);
        Ok(())
    }

    pub fn get_usdc_price(_ctx: Context<GetUsdcPrice>) -> Result<()> {
        msg!("Current USDC price: $1.00");
        Ok(())
    }

    pub fn check_emergency_stop(ctx: Context<CheckEmergencyStop>) -> Result<()> {
        let price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        let is_stopped = price_oracle.is_emergency_stopped();
        msg!("Emergency stop status: {}", is_stopped);
        Ok(())
    }

    pub fn set_emergency_stop(ctx: Context<SetEmergencyStop>, stop: bool) -> Result<()> {
        let mut price_oracle = PriceOracle {
            header: ctx.accounts.header.clone(),
            data: ctx.accounts.data.clone(),
        };
        price_oracle.set_emergency_stop(stop);
        msg!("Emergency stop set to: {}", stop);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<PriceOracleHeader>())]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<PriceOracleData>())]
    pub data: Account<'info, PriceOracleData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(mut)]
    pub data: Account<'info, PriceOracleData>,
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
}

#[derive(Accounts)]
pub struct UpdateApy<'info> {
    #[account(mut)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(mut)]
    pub data: Account<'info, PriceOracleData>,
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
}

#[derive(Accounts)]
pub struct GetCurrentPrice<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetCurrentApy<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetSolPrice<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetUsdcPrice {}

#[derive(Accounts)]
pub struct CheckEmergencyStop<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct SetEmergencyStop<'info> {
    #[account(mut)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(mut)]
    pub data: Account<'info, PriceOracleData>,
    pub authority: Signer<'info>,
}