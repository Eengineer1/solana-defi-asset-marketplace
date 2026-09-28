use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

mod state;
use state::*;

declare_id!("BDGYk9AwrWG8wKey93VzpetV8d2qhMoEbPyi89FJ9eCP");

const METADATA_PROGRAM: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

#[program]
pub mod asset_marketplace {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>, fee_bps: u16) -> Result<()> {
        require!(fee_bps <= 1_000, MarketError::FeeTooHigh);
        let market = &mut ctx.accounts.market;
        market.admin = ctx.accounts.admin.key();
        market.fee_recipient = ctx.accounts.fee_recipient.key();
        market.fee_bps = fee_bps;
        market.bump = ctx.bumps.market;
        Ok(())
    }

    pub fn create_listing(ctx: Context<CreateListing>, price: u64) -> Result<()> {
        require!(price > 0, MarketError::ZeroPrice);
        require!(ctx.accounts.asset_mint.decimals == 0, MarketError::NonNftMint);
        assert_canonical_metadata(
            &ctx.accounts.asset_mint.key(),
            &ctx.accounts.metadata.to_account_info(),
        )?;

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.asset_mint = ctx.accounts.asset_mint.key();
        listing.payment_mint = ctx.accounts.payment_mint.key();
        listing.asset_token_program = ctx.accounts.asset_token_program.key();
        listing.payment_token_program = ctx.accounts.payment_token_program.key();
        listing.escrow = ctx.accounts.escrow.key();
        listing.price = price;
        listing.bump = ctx.bumps.listing;

        let cpi = TransferChecked {
            from: ctx.accounts.seller_asset.to_account_info(),
            mint: ctx.accounts.asset_mint.to_account_info(),
            to: ctx.accounts.escrow.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        transfer_checked(
            CpiContext::new(ctx.accounts.asset_token_program.to_account_info(), cpi),
            1,
            0,
        )?;

        emit!(ListingCreated {
            listing: listing.key(),
            seller: listing.seller,
            asset_mint: listing.asset_mint,
            payment_mint: listing.payment_mint,
            price,
        });
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        let price = ctx.accounts.listing.price;
        let fee = (price as u128)
            .checked_mul(ctx.accounts.market.fee_bps as u128)
            .ok_or(MarketError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(MarketError::MathOverflow)? as u64;
        let seller_amount = price.checked_sub(fee).ok_or(MarketError::MathOverflow)?;

        transfer_payment(
            &ctx.accounts.payment_token_program,
            &ctx.accounts.buyer,
            &ctx.accounts.payment_mint,
            &ctx.accounts.buyer_payment,
            &ctx.accounts.seller_payment,
            seller_amount,
        )?;
        if fee > 0 {
            transfer_payment(
                &ctx.accounts.payment_token_program,
                &ctx.accounts.buyer,
                &ctx.accounts.payment_mint,
                &ctx.accounts.buyer_payment,
                &ctx.accounts.fee_payment,
                fee,
            )?;
        }

        let mint_key = ctx.accounts.asset_mint.key();
        let seeds: &[&[u8]] = &[b"listing", mint_key.as_ref(), &[ctx.accounts.listing.bump]];
        let cpi = TransferChecked {
            from: ctx.accounts.escrow.to_account_info(),
            mint: ctx.accounts.asset_mint.to_account_info(),
            to: ctx.accounts.buyer_asset.to_account_info(),
            authority: ctx.accounts.listing.to_account_info(),
        };
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.asset_token_program.to_account_info(),
                cpi,
                &[seeds],
            ),
            1,
            0,
        )?;

        emit!(ListingFilled {
            listing: ctx.accounts.listing.key(),
            buyer: ctx.accounts.buyer.key(),
            price,
            fee,
        });
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        let mint_key = ctx.accounts.asset_mint.key();
        let seeds: &[&[u8]] = &[b"listing", mint_key.as_ref(), &[ctx.accounts.listing.bump]];
        let cpi = TransferChecked {
            from: ctx.accounts.escrow.to_account_info(),
            mint: ctx.accounts.asset_mint.to_account_info(),
            to: ctx.accounts.seller_asset.to_account_info(),
            authority: ctx.accounts.listing.to_account_info(),
        };
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.asset_token_program.to_account_info(),
                cpi,
                &[seeds],
            ),
            1,
            0,
        )?;
        emit!(ListingCancelled {
            listing: ctx.accounts.listing.key(),
            seller: ctx.accounts.seller.key(),
        });
        Ok(())
    }
}

fn assert_canonical_metadata(mint: &Pubkey, metadata: &AccountInfo<'_>) -> Result<()> {
    let (expected, _) = Pubkey::find_program_address(
        &[b"metadata", METADATA_PROGRAM.as_ref(), mint.as_ref()],
        &METADATA_PROGRAM,
    );
    require_keys_eq!(metadata.key(), expected, MarketError::InvalidMetadataPda);
    require_keys_eq!(*metadata.owner, METADATA_PROGRAM, MarketError::InvalidMetadataOwner);
    Ok(())
}

fn transfer_payment<'info>(
    token_program: &Interface<'info, TokenInterface>,
    buyer: &Signer<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    destination: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
) -> Result<()> {
    let cpi = TransferChecked {
        from: source.to_account_info(),
        mint: mint.to_account_info(),
        to: destination.to_account_info(),
        authority: buyer.to_account_info(),
    };
    transfer_checked(
        CpiContext::new(token_program.to_account_info(), cpi),
        amount,
        mint.decimals,
    )
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub admin: SystemAccount<'info>,
    pub fee_recipient: SystemAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market"],
        bump
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(seeds = [b"market"], bump = market.bump)]
    pub market: Account<'info, Market>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub payment_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: canonical address and owner are validated in the instruction.
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", asset_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = asset_mint,
        associated_token::authority = listing,
        associated_token::token_program = asset_token_program
    )]
    pub escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = seller_asset.owner == seller.key(),
        constraint = seller_asset.mint == asset_mint.key()
    )]
    pub seller_asset: InterfaceAccount<'info, TokenAccount>,
    pub asset_token_program: Interface<'info, TokenInterface>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: receives rent when listing account closes; identity is constrained by listing.seller.
    #[account(mut, address = listing.seller)]
    pub seller: UncheckedAccount<'info>,
    #[account(seeds = [b"market"], bump = market.bump)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        close = seller,
        seeds = [b"listing", asset_mint.key().as_ref()],
        bump = listing.bump,
        has_one = asset_mint,
        has_one = payment_mint,
        has_one = escrow,
        constraint = listing.asset_token_program == asset_token_program.key(),
        constraint = listing.payment_token_program == payment_token_program.key()
    )]
    pub listing: Account<'info, Listing>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub payment_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, constraint = escrow.owner == listing.key(), constraint = escrow.mint == asset_mint.key())]
    pub escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = buyer_asset.owner == buyer.key(), constraint = buyer_asset.mint == asset_mint.key())]
    pub buyer_asset: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = buyer_payment.owner == buyer.key(), constraint = buyer_payment.mint == payment_mint.key())]
    pub buyer_payment: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = seller_payment.owner == listing.seller, constraint = seller_payment.mint == payment_mint.key())]
    pub seller_payment: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = fee_payment.owner == market.fee_recipient, constraint = fee_payment.mint == payment_mint.key())]
    pub fee_payment: InterfaceAccount<'info, TokenAccount>,
    pub asset_token_program: Interface<'info, TokenInterface>,
    pub payment_token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        close = seller,
        seeds = [b"listing", asset_mint.key().as_ref()],
        bump = listing.bump,
        has_one = seller,
        has_one = asset_mint,
        has_one = escrow,
        constraint = listing.asset_token_program == asset_token_program.key()
    )]
    pub listing: Account<'info, Listing>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, constraint = escrow.owner == listing.key(), constraint = escrow.mint == asset_mint.key())]
    pub escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = seller_asset.owner == seller.key(), constraint = seller_asset.mint == asset_mint.key())]
    pub seller_asset: InterfaceAccount<'info, TokenAccount>,
    pub asset_token_program: Interface<'info, TokenInterface>,
}

#[event]
pub struct ListingCreated {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub asset_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub price: u64,
}

#[event]
pub struct ListingFilled {
    pub listing: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
    pub fee: u64,
}

#[event]
pub struct ListingCancelled {
    pub listing: Pubkey,
    pub seller: Pubkey,
}

#[error_code]
pub enum MarketError {
    #[msg("fee exceeds the configured safety cap")]
    FeeTooHigh,
    #[msg("price must be non-zero")]
    ZeroPrice,
    #[msg("asset mint must have zero decimals")]
    NonNftMint,
    #[msg("metadata PDA is not canonical for this mint")]
    InvalidMetadataPda,
    #[msg("metadata account is not owned by the Metaplex Token Metadata program")]
    InvalidMetadataOwner,
    #[msg("arithmetic overflow")]
    MathOverflow,
}
