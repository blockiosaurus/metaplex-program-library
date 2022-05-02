//! Create both private and public bids.
//! A private bid is a bid on a specific NFT *held by a specific person*. A public bid is a bid on a specific NFT *regardless of who holds it*.

use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    //auction_house::{
    cpi::accounts::{BuyWithAuctioneer as AHBuy, PublicBuyWithAuctioneer as AHPublicBuy},
    program::AuctionHouse as AuctionHouseProgram, //program::auction_house as AuctionHouseProgram,
    //program::auction_house,
    //},
    AuctionHouse,
};

use crate::{constants::*, sell::config::*, utils::*};

/// Accounts for the [`public_bid_with_auctioneer` handler](fn.public_bid_with_auctioneer.html).
#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct AuctioneerPublicBuy<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        seeds=[
            LISTING_CONFIG.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &token_size.to_le_bytes()
        ],
        bump,
    )]
    pub listing_config: Account<'info, ListingConfig>,

    /// The seller of the NFT
    pub seller: UncheckedAccount<'info>,

    // Accounts passed into Auction House CPI call
    wallet: Signer<'info>,

    /// CHECK: Verified through CPI
    #[account(mut)]
    payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    transfer_authority: UncheckedAccount<'info>,

    treasury_mint: Box<Account<'info, Mint>>,

    token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    metadata: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], seeds::program=auction_house_program, bump = escrow_payment_bump)]
    escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified with has_one constraint on auction house account.
    authority: UncheckedAccount<'info>,

    #[account(seeds = [PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump = auction_house.bump, has_one = authority, has_one = treasury_mint, has_one = auction_house_fee_account)]
    auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump = auction_house.fee_payer_bump)]
    auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(mut, seeds = [PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), treasury_mint.key().as_ref(), token_account.mint.as_ref(), buyer_price.to_le_bytes().as_ref(), token_size.to_le_bytes().as_ref()], seeds::program=auction_house_program, bump = trade_state_bump)]
    buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Create a bid on a specific SPL token.
/// Public bids are specific to the token itself, rather than the auction, and remain open indefinitely until either the user closes it or the requirements for the bid are met and it is matched with a counter bid and closed as a transaction.
pub fn auctioneer_public_buy(
    ctx: Context<AuctioneerPublicBuy>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    assert_auction_valid(&ctx.accounts.listing_config)?;
    assert_higher_bid(&ctx.accounts.listing_config, buyer_price)?;
    ctx.accounts.listing_config.highest_bid.amount = buyer_price;
    ctx.accounts.listing_config.highest_bid.buyer_trade_state =
        ctx.accounts.buyer_trade_state.key();

    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHPublicBuy {
        wallet: ctx.accounts.wallet.to_account_info(),
        payment_account: ctx.accounts.payment_account.to_account_info(),
        transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    mpl_auction_house::cpi::public_buy_with_auctioneer(
        cpi_ctx,
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
    )
}

/// Accounts for the [`private_bid_with_auctioneer` handler](fn.private_bid_with_auctioneer.html).
#[derive(Accounts)]
#[instruction(trade_state_bump: u8, escrow_payment_bump: u8, buyer_price: u64, token_size: u64)]
pub struct AuctioneerBuy<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        seeds=[
            LISTING_CONFIG.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &token_size.to_le_bytes()
        ],
        bump,
    )]
    pub listing_config: Account<'info, ListingConfig>,

    /// The seller of the NFT
    pub seller: UncheckedAccount<'info>,

    // Accounts passed into Auction House CPI call
    /// User wallet account.
    wallet: Signer<'info>,

    /// CHECK: Verified through CPI
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    payment_account: UncheckedAccount<'info>,

    /// CHECK:
    /// SPL token account transfer authority.
    transfer_authority: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    treasury_mint: Box<Account<'info, Mint>>,

    /// SPL token account.
    token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    /// SPL token account metadata.
    metadata: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ], seeds::program=auction_house_program,
        bump = escrow_payment_bump
    )]
    escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified with has_one constraint on auction house account.
    /// Auction House instance authority account.
    authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds = [PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump = auction_house.bump, has_one = authority, has_one = treasury_mint, has_one = auction_house_fee_account)]
    auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds = [PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump = auction_house.fee_payer_bump)]
    auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer trade state PDA.
    #[account(mut, seeds = [PREFIX.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), treasury_mint.key().as_ref(), token_account.mint.as_ref(), buyer_price.to_le_bytes().as_ref(), token_size.to_le_bytes().as_ref()], seeds::program=auction_house_program, bump = trade_state_bump)]
    buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Is used as a seed for ah_auctioneer_pda.
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ], seeds::program=auction_house_program,
        bump = auction_house.auctioneer_pda_bump,
    )]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Create a private bid on a specific SPL token that is *held by a specific wallet*.
pub fn auctioneer_buy<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerBuy<'info>>,
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    assert_auction_valid(&ctx.accounts.listing_config)?;
    assert_higher_bid(&ctx.accounts.listing_config, buyer_price)?;
    ctx.accounts.listing_config.highest_bid.amount = buyer_price;
    ctx.accounts.listing_config.highest_bid.buyer_trade_state =
        ctx.accounts.buyer_trade_state.key();

    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHBuy {
        wallet: ctx.accounts.wallet.to_account_info(),
        payment_account: ctx.accounts.payment_account.to_account_info(),
        transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    mpl_auction_house::cpi::buy_with_auctioneer(
        cpi_ctx,
        trade_state_bump,
        escrow_payment_bump,
        buyer_price,
        token_size,
    )
}
