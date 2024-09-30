use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::log::sol_log_compute_units;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

pub mod price_oracle;
use price_oracle::{AssetType, PriceOracle, PriceOracleHeader, PriceOracleData, OracleError};

declare_id!("9rK2cdpDThj5s3qa4W6FA7D4E3gnXjVxBTgen8FypMUj");

// Define price change threshold constant
const PRICE_CHANGE_THRESHOLD: f64 = 0.20; // 20%

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

    pub fn update_price_and_apy(ctx: Context<UpdatePriceAndApy>, asset_type: AssetType) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating price and APY for {:?}", asset_type);

        let clock = Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?;

        // Validate Switchboard program ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            msg!("Invalid Switchboard account owner: expected {}, found {}", 
                ctx.accounts.header.switchboard_program_id, 
                ctx.accounts.oracle_feed.to_account_info().owner);
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let old_price = PriceOracle::get_current_price(&ctx.accounts.data, asset_type)
            .unwrap_or_else(|_| 0.0);
        let old_apy = PriceOracle::get_current_apy(&ctx.accounts.data, asset_type)
            .unwrap_or_else(|_| 0.0);

        PriceOracle::update_price_and_apy(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            asset_type,
            &clock,
        )?;

        let new_price = PriceOracle::get_current_price(&ctx.accounts.data, asset_type)?;
        let new_apy = PriceOracle::get_current_apy(&ctx.accounts.data, asset_type)?;
        
        msg!("Updated price for {:?}: {} -> {}", asset_type, old_price, new_price);
        msg!("Updated APY for {:?}: {} -> {}", asset_type, old_apy, new_apy);
        
        let price_change = if old_price != 0.0 { (new_price - old_price).abs() / old_price } else { 1.0 };
        if price_change > PRICE_CHANGE_THRESHOLD {
            msg!("Warning: Price change exceeds {}%", PRICE_CHANGE_THRESHOLD * 100.0);
        }

        sol_log_compute_units();
        Ok(())
    }

    pub fn update_sol_price(ctx: Context<UpdateSolPrice>) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating SOL price");

        let clock = Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?;

        // Validate Switchboard program ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            msg!("Invalid Switchboard account owner: expected {}, found {}", 
                ctx.accounts.header.switchboard_program_id, 
                ctx.accounts.oracle_feed.to_account_info().owner);
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

        let old_price = PriceOracle::get_current_price(&ctx.accounts.data, AssetType::SOL)
            .unwrap_or_else(|_| 0.0);

        PriceOracle::update_sol_price(
            &mut ctx.accounts.header,
            &mut ctx.accounts.data,
            &ctx.accounts.oracle_feed,
            &clock,
        )?;

        let new_price = PriceOracle::get_current_price(&ctx.accounts.data, AssetType::SOL)?;
        msg!("Updated SOL price: {} -> {}", old_price, new_price);

        let price_change = if old_price != 0.0 { (new_price - old_price).abs() / old_price } else { 1.0 };
        if price_change > PRICE_CHANGE_THRESHOLD {
            msg!("Warning: SOL price change exceeds {}%", PRICE_CHANGE_THRESHOLD * 100.0);
        }

        sol_log_compute_units();
        Ok(())
    }

    pub fn update_switchboard_program_id(ctx: Context<UpdateSwitchboardProgramId>, new_program_id: Pubkey) -> Result<()> {
        msg!("Updating Switchboard program ID from {} to {}", ctx.accounts.header.switchboard_program_id, new_program_id);
        ctx.accounts.header.switchboard_program_id = new_program_id;
        msg!("Updated Switchboard program ID to: {}", new_program_id);
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
pub struct UpdatePriceAndApy<'info> {
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
    #[account()]
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
    #[account()]
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    #[account(constraint = authority.key() == header.authority @ OracleError::UnauthorizedAccess)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateSwitchboardProgramId<'info> {
    #[account(
        mut,
        seeds = [PriceOracle::HEADER_SEED],
        bump = header.bump,
    )]
    pub header: Account<'info, PriceOracleHeader>,
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