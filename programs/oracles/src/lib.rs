use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::log::sol_log_compute_units;
use switchboard_v2::AggregatorAccountData;

pub mod price_oracle;
pub mod switchboard_utils;

use price_oracle::{AssetType, PriceOracle, PriceOracleHeader, PriceOracleData, OracleError};
use switchboard_utils::{DEVNET_AGGREGATOR_PUBKEY, SOL_PRICE_AGGREGATOR_PUBKEY};

declare_id!("GqYaWFTAy3dTNZ8zRb9EyWLqTQ4gRHUUwCCuD5GmRihY");

#[program]
pub mod oracles {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, switchboard_program_id: Pubkey) -> Result<()> {
        msg!("Initializing Price Oracle");
        PriceOracle::initialize(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.authority,
            switchboard_program_id,
            *ctx.bumps.get("header").unwrap(),
            *ctx.bumps.get("data").unwrap(),
        )?;
        msg!("Price oracle initialized successfully");
        Ok(())
    }

    pub fn update_prices_and_apys(ctx: Context<UpdatePricesAndApys>) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating prices and APYs for all assets");

        let clock = Clock::get().unwrap();

        // Validate Switchboard program ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            msg!("Invalid Switchboard account owner: expected {}, found {}", 
                ctx.accounts.header.switchboard_program_id, 
                ctx.accounts.oracle_feed.to_account_info().owner);
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        PriceOracle::update_prices_and_apys(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            &clock,
        )?;

        msg!("Prices and APYs updated successfully");
        sol_log_compute_units();
        Ok(())
    }

    pub fn update_sol_price(ctx: Context<UpdateSolPrice>) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating SOL price");

        let clock = Clock::get().unwrap();

        // Validate Switchboard program ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            msg!("Invalid Switchboard account owner: expected {}, found {}", 
                ctx.accounts.header.switchboard_program_id, 
                ctx.accounts.oracle_feed.to_account_info().owner);
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        PriceOracle::update_sol_price(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            &clock,
        )?;

        msg!("SOL price updated successfully");
        sol_log_compute_units();
        Ok(())
    }

    pub fn get_current_price(ctx: Context<GetPrice>, asset_type: AssetType) -> Result<()> {
        let price = PriceOracle::get_current_price(&ctx.accounts.data, asset_type)?;
        msg!("Current price for {:?}: {}", asset_type, price);
        Ok(())
    }

    pub fn get_current_apy(ctx: Context<GetApy>, asset_type: AssetType) -> Result<()> {
        let apy = PriceOracle::get_current_apy(&ctx.accounts.data, asset_type)?;
        msg!("Current APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn set_emergency_stop(ctx: Context<SetEmergencyStop>, stop: bool) -> Result<()> {
        PriceOracle::set_emergency_stop(&mut ctx.accounts.header, stop);
        msg!("Emergency stop set to: {}", stop);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<PriceOracleHeader>(),
        seeds = [PriceOracle::HEADER_SEED],
        bump
    )]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<PriceOracleData>(),
        seeds = [PriceOracle::DATA_SEED],
        bump
    )]
    pub data: Account<'info, PriceOracleData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePricesAndApys<'info> {
    #[account(
        mut,
        seeds = [PriceOracle::HEADER_SEED],
        bump = header.bump,
    )]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(
        mut,
        seeds = [PriceOracle::DATA_SEED],
        bump = data.bump,
    )]
    pub data: Account<'info, PriceOracleData>,
    #[account(
        constraint = oracle_feed.key() == DEVNET_AGGREGATOR_PUBKEY.parse::<Pubkey>().unwrap()
    )]
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateSolPrice<'info> {
    #[account(
        mut,
        seeds = [PriceOracle::HEADER_SEED],
        bump = header.bump,
    )]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(
        mut,
        seeds = [PriceOracle::DATA_SEED],
        bump = data.bump,
    )]
    pub data: Account<'info, PriceOracleData>,
    #[account(
        constraint = oracle_feed.key() == SOL_PRICE_AGGREGATOR_PUBKEY.parse::<Pubkey>().unwrap()
    )]
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    #[account(
        seeds = [PriceOracle::DATA_SEED],
        bump = data.bump,
    )]
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetApy<'info> {
    #[account(
        seeds = [PriceOracle::DATA_SEED],
        bump = data.bump,
    )]
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct SetEmergencyStop<'info> {
    #[account(
        mut,
        seeds = [PriceOracle::HEADER_SEED],
        bump = header.bump,
    )]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}