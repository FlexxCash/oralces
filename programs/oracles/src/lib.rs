use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::log::sol_log_compute_units;
use switchboard_v2::AggregatorAccountData;

pub mod price_oracle;
use price_oracle::{AssetType, AssetTypeWrapper, PriceOracle, PriceOracleHeader, PriceOracleData, OracleError, convert_switchboard_decimal};

declare_id!("BPmubr9zibqNErffzaxBchZcrMUwhXaNw8VBpKKCkoob");

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

    pub fn update_price(ctx: Context<UpdatePrice>, asset_type: AssetType) -> Result<()> {
        sol_log_compute_units();
        msg!("Updating price for {:?}", asset_type);

        let clock = Clock::get().map_err(|_| error!(OracleError::ClockUnavailable))?;

        // 驗證 Switchboard 程式 ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

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

        // 驗證 Switchboard 程式 ID
        if ctx.accounts.oracle_feed.to_account_info().owner != &ctx.accounts.header.switchboard_program_id {
            return Err(error!(OracleError::InvalidSwitchboardAccount));
        }

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

    // ... 其他函數保持不變 ...

    pub fn update_switchboard_program_id(ctx: Context<UpdateSwitchboardProgramId>, new_program_id: Pubkey) -> Result<()> {
        ctx.accounts.header.switchboard_program_id = new_program_id;
        msg!("Updated Switchboard program ID to: {}", new_program_id);
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
    #[account()]
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

// ... 其他結構體保持不變 ...