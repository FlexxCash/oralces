use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::log::sol_log_compute_units;
use switchboard_v2::{AggregatorAccountData, SWITCHBOARD_PROGRAM_ID};

pub mod price_oracle;
use price_oracle::{AssetType, AssetTypeWrapper, PriceOracle, PriceOracleHeader, PriceOracleData, OracleError, convert_switchboard_decimal};

declare_id!("GxkpGSztczkz7hNPUcN8XbZjnyMYqW8YMmTqtKVA579e");

#[program]
pub mod oracles {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing Price Oracle");
        PriceOracle::initialize(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.authority,
            *ctx.bumps.get("header").unwrap(),
            *ctx.bumps.get("data").unwrap(),
        )?;
        msg!("Price oracle initialized successfully");
        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, asset_type: AssetType) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating price for {:?}", asset_type);

        let clock = Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?;

        // 從 Switchboard 聚合器獲取最新價格
        let aggregator = &ctx.accounts.oracle_feed.load()?;
        let switchboard_decimal = aggregator.get_result()?;
        let latest_price = convert_switchboard_decimal(switchboard_decimal)
            .map_err(|_| error!(OracleError::SwitchboardConversionError))?;

        PriceOracle::update_price(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            asset_type,
            &clock,
        )?;

        let new_price = PriceOracle::get_current_price(&ctx.accounts.data, asset_type)?;
        msg!("Updated price for {:?}: {}", asset_type, new_price);
        sol_log_compute_units();
        Ok(())
    }

    pub fn update_apy(ctx: Context<UpdateApy>, asset_type: AssetType) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating APY for {:?}", asset_type);

        let clock = Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?;

        PriceOracle::update_apy(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            asset_type,
            &clock,
        )?;

        let apy = PriceOracle::get_current_apy(&ctx.accounts.data, asset_type)?;
        msg!("Updated APY for {:?}: {}", asset_type, apy);
        sol_log_compute_units();
        Ok(())
    }

    pub fn get_current_price(ctx: Context<GetCurrentPrice>, asset_type: AssetType) -> Result<()> {
        let price = PriceOracle::get_current_price(&ctx.accounts.data, asset_type)?;
        msg!("Current price for {:?}: {}", asset_type, price);
        Ok(())
    }

    pub fn get_current_apy(ctx: Context<GetCurrentApy>, asset_type: AssetType) -> Result<()> {
        let apy = PriceOracle::get_current_apy(&ctx.accounts.data, asset_type)?;
        msg!("Current APY for {:?}: {}", asset_type, apy);
        Ok(())
    }

    pub fn get_sol_price(ctx: Context<GetSolPrice>) -> Result<()> {
        let sol_price = PriceOracle::get_current_price(&ctx.accounts.data, AssetType::SOL)?;
        msg!("Current SOL price: {}", sol_price);
        Ok(())
    }

    pub fn get_usdc_price(_ctx: Context<GetUsdcPrice>) -> Result<()> {
        msg!("Current USDC price: $1.00");
        Ok(())
    }

    pub fn check_emergency_stop(ctx: Context<CheckEmergencyStop>) -> Result<()> {
        let is_stopped = PriceOracle::is_emergency_stopped(&ctx.accounts.header);
        msg!("Emergency stop status: {}", is_stopped);
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
pub struct UpdatePrice<'info> {
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
        constraint = oracle_feed.to_account_info().owner == &SWITCHBOARD_PROGRAM_ID @ OracleError::InvalidSwitchboardAccount
    )]
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateApy<'info> {
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
        constraint = oracle_feed.to_account_info().owner == &SWITCHBOARD_PROGRAM_ID @ OracleError::InvalidSwitchboardAccount
    )]
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    /// CHECK: This is the Switchboard program ID
    #[account(address = SWITCHBOARD_PROGRAM_ID @ OracleError::InvalidSwitchboardProgram)]
    pub switchboard_program: AccountInfo<'info>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetCurrentPrice<'info> {
    #[account(seeds = [PriceOracle::HEADER_SEED], bump = header.bump)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(seeds = [PriceOracle::DATA_SEED], bump = data.bump)]
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetCurrentApy<'info> {
    #[account(seeds = [PriceOracle::HEADER_SEED], bump = header.bump)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(seeds = [PriceOracle::DATA_SEED], bump = data.bump)]
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetSolPrice<'info> {
    #[account(seeds = [PriceOracle::HEADER_SEED], bump = header.bump)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(seeds = [PriceOracle::DATA_SEED], bump = data.bump)]
    pub data: Account<'info, PriceOracleData>,
}

#[derive(Accounts)]
pub struct GetUsdcPrice {}

#[derive(Accounts)]
pub struct CheckEmergencyStop<'info> {
    #[account(seeds = [PriceOracle::HEADER_SEED], bump = header.bump)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(seeds = [PriceOracle::DATA_SEED], bump = data.bump)]
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
    #[account(
        mut,
        seeds = [PriceOracle::DATA_SEED],
        bump = data.bump,
    )]
    pub data: Account<'info, PriceOracleData>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}