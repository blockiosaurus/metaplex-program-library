pub mod bid;
pub mod cancel;
pub mod constants;
pub mod deposit;
pub mod errors;
pub mod execute_sale;
pub mod pda;
pub mod sell;
pub mod utils;
pub mod withdraw;

use crate::{bid::*, cancel::*, deposit::*, execute_sale::*, sell::*, withdraw::*};

use anchor_lang::prelude::*;

use solana_program::clock::UnixTimestamp;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

//pub const PREFIX: &str = "auctioneer";

#[program]
pub mod auctioneer {
    use super::*;

    /// Withdraw `amount` from the escrow payment account for your specific wallet.
    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerWithdraw<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> Result<()> {
        auctioneer_withdraw(ctx, escrow_payment_bump, amount)
    }

    /// Deposit `amount` into the escrow payment account for your specific wallet.
    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> Result<()> {
        auctioneer_deposit(ctx, escrow_payment_bump, amount)
    }

    /// Cancel a bid or ask by revoking the token delegate, transferring all lamports from the trade state account to the fee payer, and setting the trade state account data to zero so it can be garbage collected.
    pub fn cancel<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerCancel<'info>>,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_cancel(ctx, buyer_price, token_size)
    }

    /// Execute sale between provided buyer and seller trade state accounts transferring funds to seller wallet and token to buyer wallet.
    #[inline(never)]
    pub fn execute_sale<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerExecuteSale<'info>>,
        escrow_payment_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_execute_sale(
            ctx,
            escrow_payment_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
        )
    }

    /// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
    pub fn sell<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerSell<'info>>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
        start_time: UnixTimestamp,
        end_time: UnixTimestamp,
    ) -> Result<()> {
        auctioneer_sell(
            ctx,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
            start_time,
            end_time,
        )
    }

    /// Create a private buy bid by creating a `buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerBuy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_buy(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
        )
    }

    /// Create a public buy bid by creating a `public_buyer_trade_state` account and an `escrow_payment` account and funding the escrow with the necessary SOL or SPL token amount.
    pub fn public_buy<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerPublicBuy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_public_buy(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
        )
    }
}
