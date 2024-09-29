use anchor_lang::prelude::*;
use switchboard_v2::AggregatorAccountData;
use crate::price_oracle::{PriceOracleHeader, PriceData};

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(mut)]
    pub data: Account<'info, PriceData>,
    pub oracle_feed: AccountLoader<'info, AggregatorAccountData>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub header: Account<'info, PriceOracleHeader>,
    pub data: Account<'info, PriceData>,
}

#[derive(Accounts)]
pub struct UpdateApy<'info> {
    #[account(mut)]
    pub header: Account<'info, PriceOracleHeader>,
    #[account(mut)]
    pub data: Account<'info, PriceData>,
    #[account(mut)]
    pub authority: Signer<'info>,
}