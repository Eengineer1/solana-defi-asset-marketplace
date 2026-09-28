use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub seller: Pubkey,
    pub asset_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub asset_token_program: Pubkey,
    pub payment_token_program: Pubkey,
    pub escrow: Pubkey,
    pub price: u64,
    pub bump: u8,
}
