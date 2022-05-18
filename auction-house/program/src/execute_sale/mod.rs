use anchor_lang::{prelude::*, AnchorDeserialize};
use solana_program::program_memory::sol_memset;

use crate::{constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`execute_sale` handler](auction_house/fn.execute_sale.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, buyer_price: u64, token_size: u64)]
pub struct ExecuteSale<'info> {
    /// CHECK: Verified through CPI
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Token mint account for the SPL token.
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    /// Auction House treasury mint account.
    pub treasury_mint: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), buyer.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_treasury, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump=auction_house.treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &buyer_price.to_le_bytes(), &token_size.to_le_bytes()], bump=seller_trade_state.to_account_info().data.borrow()[0])]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], bump=free_trade_state_bump)]
    pub free_trade_state: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

// impl<'info> From<ExecuteSaleWithAuctioneer<'info>> for ExecuteSale<'info> {
//     fn from(a: ExecuteSaleWithAuctioneer<'info>) -> ExecuteSale<'info> {
//         ExecuteSale {
//             buyer: a.buyer,
//             seller: a.seller,
//             token_account: a.token_account,
//             token_mint: a.token_mint,
//             metadata: a.metadata,
//             treasury_mint: a.treasury_mint,
//             escrow_payment_account: a.escrow_payment_account,
//             seller_payment_receipt_account: a.seller_payment_receipt_account,
//             buyer_receipt_token_account: a.buyer_receipt_token_account,
//             authority: a.authority,
//             auction_house: a.auction_house.into(),
//             auction_house_fee_account: a.auction_house_fee_account,
//             auction_house_treasury: a.auction_house_treasury,
//             buyer_trade_state: a.buyer_trade_state,
//             seller_trade_state: a.seller_trade_state,
//             free_trade_state: a.free_trade_state,
//             token_program: a.token_program,
//             system_program: a.system_program,
//             ata_program: a.ata_program,
//             program_as_signer: a.program_as_signer,
//             rent: a.rent,
//         }
//     }
// }

pub fn execute_sale<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSale<'info>>,
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use *_with_auctioneer handler.
    if auction_house.has_auctioneer {
        return Err(AuctionHouseError::MustUseAuctioneerHandler.into());
    }

    execute_sale_logic(
        ctx,
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
        token_size,
    )
}

#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8, free_trade_state_bump: u8, program_as_signer_bump: u8, buyer_price: u64, token_size: u64)]
pub struct ExecuteSaleWithAuctioneer<'info> {
    /// CHECK: Verified through CPI
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Token mint account for the SPL token.
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    // cannot mark these as real Accounts or else we blow stack size limit
    /// Auction House treasury mint account.
    pub treasury_mint: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(mut, seeds=[PREFIX.as_bytes(), bidding_auction_house.key().as_ref(), buyer.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority.
    pub authority: UncheckedAccount<'info>,

    /// Listing Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), listing_auction_house.creator.as_ref(), listing_auction_house.treasury_mint.as_ref()], bump=listing_auction_house.bump/*, has_one=authority*/, has_one=treasury_mint/*, has_one=listing_auction_house_treasury, has_one=auction_house_fee_account*/)]
    pub listing_auction_house: Box<Account<'info, AuctionHouse>>,

    /// Buying Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), bidding_auction_house.creator.as_ref(), bidding_auction_house.treasury_mint.as_ref()], bump=bidding_auction_house.bump/*, has_one=authority*/, has_one=treasury_mint,/* has_one=bidding_auction_house_treasury, has_one=auction_house_fee_account*/)]
    pub bidding_auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), bidding_auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=bidding_auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(mut, seeds=[PREFIX.as_bytes(), listing_auction_house.key().as_ref(), TREASURY.as_bytes()], bump=listing_auction_house.treasury_bump)]
    pub listing_auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(mut, seeds=[PREFIX.as_bytes(), bidding_auction_house.key().as_ref(), TREASURY.as_bytes()], bump=bidding_auction_house.treasury_bump)]
    pub bidding_auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), listing_auction_house.key().as_ref(), token_account.key().as_ref(), listing_auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &u64::MAX.to_le_bytes(), &token_size.to_le_bytes()], bump=seller_trade_state.to_account_info().data.borrow()[0])]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(mut, seeds=[PREFIX.as_bytes(), seller.key().as_ref(), listing_auction_house.key().as_ref(), token_account.key().as_ref(), listing_auction_house.treasury_mint.as_ref(), token_mint.key().as_ref(), &0u64.to_le_bytes(), &token_size.to_le_bytes()], bump=free_trade_state_bump)]
    pub free_trade_state: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), bidding_auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = bidding_auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn execute_sale_with_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSaleWithAuctioneer<'info>>,
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let listing_auction_house = &ctx.accounts.listing_auction_house;
    let bidding_auction_house = &ctx.accounts.bidding_auction_house;
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let ah_auctioneer_pda = &ctx.accounts.ah_auctioneer_pda;

    if !bidding_auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        &bidding_auction_house.key(),
        &auctioneer_authority.key(),
        ah_auctioneer_pda,
        AuthorityScope::ExecuteSale,
    )?;

    // Duplicate the logic methods to avoid going over the compute limit.
    execute_auction_sale_logic(
        ctx,
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
        token_size,
    )
}

/// Execute sale between provided buyer and seller trade state accounts transferring funds to seller wallet and token to buyer wallet.
#[inline(never)]
fn execute_auction_sale_logic<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSaleWithAuctioneer<'info>>,
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let buyer = &ctx.accounts.buyer;
    let seller = &ctx.accounts.seller;
    let token_account = &ctx.accounts.token_account;
    let token_mint = &ctx.accounts.token_mint;
    let metadata = &ctx.accounts.metadata;
    let treasury_mint = &ctx.accounts.treasury_mint;
    let seller_payment_receipt_account = &ctx.accounts.seller_payment_receipt_account;
    let buyer_receipt_token_account = &ctx.accounts.buyer_receipt_token_account;
    let escrow_payment_account = &ctx.accounts.escrow_payment_account;
    let authority = &ctx.accounts.authority;
    let bidding_auction_house = &ctx.accounts.bidding_auction_house;
    let listing_auction_house = &ctx.accounts.listing_auction_house;
    let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
    let auction_house_treasury = &ctx.accounts.bidding_auction_house_treasury;
    let buyer_trade_state = &ctx.accounts.buyer_trade_state;
    let seller_trade_state = &ctx.accounts.seller_trade_state;
    let free_trade_state = &ctx.accounts.free_trade_state;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let ata_program = &ctx.accounts.ata_program;
    let program_as_signer = &ctx.accounts.program_as_signer;
    let rent = &ctx.accounts.rent;

    let metadata_clone = metadata.to_account_info();
    let escrow_clone = escrow_payment_account.to_account_info();
    let auction_house_clone = bidding_auction_house.to_account_info();
    let ata_clone = ata_program.to_account_info();
    let token_clone = token_program.to_account_info();
    let sys_clone = system_program.to_account_info();
    let rent_clone = rent.to_account_info();
    let treasury_clone = auction_house_treasury.to_account_info();
    let authority_clone = authority.to_account_info();
    let buyer_receipt_clone = buyer_receipt_token_account.to_account_info();
    let token_account_clone = token_account.to_account_info();

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    if buyer_price == 0 && !authority_clone.is_signer && !seller.is_signer {
        return Err(
            AuctionHouseError::CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff.into(),
        );
    }

    let token_account_mint = get_mint_from_token_account(&token_account_clone)?;

    assert_keys_equal(token_mint.key(), token_account_mint)?;
    let delegate = get_delegate_from_token_account(&token_account_clone)?;
    if let Some(d) = delegate {
        assert_keys_equal(program_as_signer.key(), d)?;
    } else {
        msg!("No delegate detected on token account.");
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }
    let buyer_ts_data = &mut buyer_trade_state.try_borrow_mut_data()?;
    let seller_ts_data = &mut seller_trade_state.try_borrow_mut_data()?;
    let ts_bump = buyer_ts_data[0];
    msg!("Here 0");
    assert_valid_trade_state(
        &buyer.key(),
        bidding_auction_house,
        buyer_price,
        token_size,
        buyer_trade_state,
        &token_mint.key(),
        &token_account.key(),
        ts_bump,
    )?;
    msg!("Here 1");
    if ts_bump == 0 || buyer_ts_data.len() == 0 || seller_ts_data.len() == 0 {
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }

    let auction_house_key = bidding_auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[bidding_auction_house.fee_payer_bump],
    ];

    let wallet_to_use = if buyer.is_signer { buyer } else { seller };

    let (fee_payer, fee_payer_seeds) = get_fee_payer(
        authority,
        bidding_auction_house,
        wallet_to_use.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    let fee_payer_clone = fee_payer.to_account_info();

    assert_is_ata(
        &token_account.to_account_info(),
        &seller.key(),
        &token_account_mint,
    )?;
    msg!("Here 2");
    assert_derivation(
        &mpl_token_metadata::id(),
        &metadata.to_account_info(),
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            token_account_mint.as_ref(),
        ],
    )?;

    msg!("Here 3");

    if metadata.data_is_empty() {
        return Err(AuctionHouseError::MetadataDoesntExist.into());
    }

    let auction_house_key = bidding_auction_house.key();
    let wallet_key = buyer.key();
    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let ah_seeds = [
        PREFIX.as_bytes(),
        bidding_auction_house.creator.as_ref(),
        bidding_auction_house.treasury_mint.as_ref(),
        &[bidding_auction_house.bump],
    ];

    // with the native account, the escrow is its own owner,
    // whereas with token, it is the auction house that is owner.
    let signer_seeds_for_royalties = if is_native {
        escrow_signer_seeds
    } else {
        ah_seeds
    };

    let buyer_leftover_after_royalties = pay_creator_fees(
        &mut ctx.remaining_accounts.iter(),
        &metadata_clone,
        &escrow_clone,
        &auction_house_clone,
        &fee_payer_clone,
        treasury_mint,
        &ata_clone,
        &token_clone,
        &sys_clone,
        &rent_clone,
        &signer_seeds_for_royalties,
        &fee_payer_seeds,
        buyer_price,
        is_native,
    )?;

    let auction_house_fee_paid = pay_auction_house_fees(
        &bidding_auction_house,
        &treasury_clone,
        &escrow_clone,
        &token_clone,
        &sys_clone,
        &signer_seeds_for_royalties,
        buyer_price,
        is_native,
    )?;

    let buyer_leftover_after_royalties_and_house_fee = buyer_leftover_after_royalties
        .checked_sub(auction_house_fee_paid)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    if !is_native {
        if seller_payment_receipt_account.data_is_empty() {
            make_ata(
                seller_payment_receipt_account.to_account_info(),
                seller.to_account_info(),
                treasury_mint.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &fee_payer_seeds,
            )?;
        }

        let seller_rec_acct = assert_is_ata(
            &seller_payment_receipt_account.to_account_info(),
            &seller.key(),
            &treasury_mint.key(),
        )?;

        // make sure you cant get rugged
        if seller_rec_acct.delegate.is_some() {
            return Err(AuctionHouseError::SellerATACannotHaveDelegate.into());
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &escrow_payment_account.key(),
                &seller_payment_receipt_account.key(),
                &bidding_auction_house.key(),
                &[],
                buyer_leftover_after_royalties_and_house_fee,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                token_program.to_account_info(),
                bidding_auction_house.to_account_info(),
            ],
            &[&ah_seeds],
        )?;
    } else {
        assert_keys_equal(seller_payment_receipt_account.key(), seller.key())?;
        invoke_signed(
            &system_instruction::transfer(
                &escrow_payment_account.key,
                seller_payment_receipt_account.key,
                buyer_leftover_after_royalties_and_house_fee,
            ),
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
    }

    if buyer_receipt_token_account.data_is_empty() {
        make_ata(
            buyer_receipt_token_account.to_account_info(),
            buyer.to_account_info(),
            token_mint.to_account_info(),
            fee_payer.to_account_info(),
            ata_program.to_account_info(),
            token_program.to_account_info(),
            system_program.to_account_info(),
            rent.to_account_info(),
            &fee_payer_seeds,
        )?;
    }

    let buyer_rec_acct = assert_is_ata(&buyer_receipt_clone, &buyer.key(), &token_mint.key())?;

    // make sure you cant get rugged
    if buyer_rec_acct.delegate.is_some() {
        return Err(AuctionHouseError::BuyerATACannotHaveDelegate.into());
    }

    let program_as_signer_seeds = [
        PREFIX.as_bytes(),
        SIGNER.as_bytes(),
        &[program_as_signer_bump],
    ];

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            &token_account.key(),
            &buyer_receipt_token_account.key(),
            &program_as_signer.key(),
            &[],
            token_size,
        )?,
        &[
            token_account.to_account_info(),
            buyer_receipt_clone,
            program_as_signer.to_account_info(),
            token_clone,
        ],
        &[&program_as_signer_seeds],
    )?;

    let curr_seller_lamp = seller_trade_state.lamports();
    **seller_trade_state.lamports.borrow_mut() = 0;
    sol_memset(&mut *seller_ts_data, 0, TRADE_STATE_SIZE);

    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_seller_lamp)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    let curr_buyer_lamp = buyer_trade_state.lamports();
    **buyer_trade_state.lamports.borrow_mut() = 0;
    sol_memset(&mut *buyer_ts_data, 0, TRADE_STATE_SIZE);
    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_buyer_lamp)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    if free_trade_state.lamports() > 0 {
        let curr_buyer_lamp = free_trade_state.lamports();
        **free_trade_state.lamports.borrow_mut() = 0;

        **fee_payer.lamports.borrow_mut() = fee_payer
            .lamports()
            .checked_add(curr_buyer_lamp)
            .ok_or(AuctionHouseError::NumericalOverflow)?;
        sol_memset(
            *free_trade_state.try_borrow_mut_data()?,
            0,
            TRADE_STATE_SIZE,
        );
    }
    Ok(())
}

/// Execute sale between provided buyer and seller trade state accounts transferring funds to seller wallet and token to buyer wallet.
#[inline(never)]
fn execute_sale_logic<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSale<'info>>,
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let buyer = &ctx.accounts.buyer;
    let seller = &ctx.accounts.seller;
    let token_account = &ctx.accounts.token_account;
    let token_mint = &ctx.accounts.token_mint;
    let metadata = &ctx.accounts.metadata;
    let treasury_mint = &ctx.accounts.treasury_mint;
    let seller_payment_receipt_account = &ctx.accounts.seller_payment_receipt_account;
    let buyer_receipt_token_account = &ctx.accounts.buyer_receipt_token_account;
    let escrow_payment_account = &ctx.accounts.escrow_payment_account;
    let authority = &ctx.accounts.authority;
    let auction_house = &ctx.accounts.auction_house;
    let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
    let auction_house_treasury = &ctx.accounts.auction_house_treasury;
    let buyer_trade_state = &ctx.accounts.buyer_trade_state;
    let seller_trade_state = &ctx.accounts.seller_trade_state;
    let free_trade_state = &ctx.accounts.free_trade_state;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let ata_program = &ctx.accounts.ata_program;
    let program_as_signer = &ctx.accounts.program_as_signer;
    let rent = &ctx.accounts.rent;

    let metadata_clone = metadata.to_account_info();
    let escrow_clone = escrow_payment_account.to_account_info();
    let auction_house_clone = auction_house.to_account_info();
    let ata_clone = ata_program.to_account_info();
    let token_clone = token_program.to_account_info();
    let sys_clone = system_program.to_account_info();
    let rent_clone = rent.to_account_info();
    let treasury_clone = auction_house_treasury.to_account_info();
    let authority_clone = authority.to_account_info();
    let buyer_receipt_clone = buyer_receipt_token_account.to_account_info();
    let token_account_clone = token_account.to_account_info();

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    if buyer_price == 0 && !authority_clone.is_signer && !seller.is_signer {
        return Err(
            AuctionHouseError::CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff.into(),
        );
    }

    let token_account_mint = get_mint_from_token_account(&token_account_clone)?;

    assert_keys_equal(token_mint.key(), token_account_mint)?;
    let delegate = get_delegate_from_token_account(&token_account_clone)?;
    if let Some(d) = delegate {
        assert_keys_equal(program_as_signer.key(), d)?;
    } else {
        msg!("No delegate detected on token account.");
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }
    let buyer_ts_data = &mut buyer_trade_state.try_borrow_mut_data()?;
    let seller_ts_data = &mut seller_trade_state.try_borrow_mut_data()?;
    let ts_bump = buyer_ts_data[0];
    assert_valid_trade_state(
        &buyer.key(),
        auction_house,
        buyer_price,
        token_size,
        buyer_trade_state,
        &token_mint.key(),
        &token_account.key(),
        ts_bump,
    )?;
    if ts_bump == 0 || buyer_ts_data.len() == 0 || seller_ts_data.len() == 0 {
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let wallet_to_use = if buyer.is_signer { buyer } else { seller };

    let (fee_payer, fee_payer_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet_to_use.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    let fee_payer_clone = fee_payer.to_account_info();

    assert_is_ata(
        &token_account.to_account_info(),
        &seller.key(),
        &token_account_mint,
    )?;
    assert_derivation(
        &mpl_token_metadata::id(),
        &metadata.to_account_info(),
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            token_account_mint.as_ref(),
        ],
    )?;

    if metadata.data_is_empty() {
        return Err(AuctionHouseError::MetadataDoesntExist.into());
    }

    let auction_house_key = auction_house.key();
    let wallet_key = buyer.key();
    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let ah_seeds = [
        PREFIX.as_bytes(),
        auction_house.creator.as_ref(),
        auction_house.treasury_mint.as_ref(),
        &[auction_house.bump],
    ];

    // with the native account, the escrow is its own owner,
    // whereas with token, it is the auction house that is owner.
    let signer_seeds_for_royalties = if is_native {
        escrow_signer_seeds
    } else {
        ah_seeds
    };

    let buyer_leftover_after_royalties = pay_creator_fees(
        &mut ctx.remaining_accounts.iter(),
        &metadata_clone,
        &escrow_clone,
        &auction_house_clone,
        &fee_payer_clone,
        treasury_mint,
        &ata_clone,
        &token_clone,
        &sys_clone,
        &rent_clone,
        &signer_seeds_for_royalties,
        &fee_payer_seeds,
        buyer_price,
        is_native,
    )?;

    let auction_house_fee_paid = pay_auction_house_fees(
        &auction_house,
        &treasury_clone,
        &escrow_clone,
        &token_clone,
        &sys_clone,
        &signer_seeds_for_royalties,
        buyer_price,
        is_native,
    )?;

    let buyer_leftover_after_royalties_and_house_fee = buyer_leftover_after_royalties
        .checked_sub(auction_house_fee_paid)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    if !is_native {
        if seller_payment_receipt_account.data_is_empty() {
            make_ata(
                seller_payment_receipt_account.to_account_info(),
                seller.to_account_info(),
                treasury_mint.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &fee_payer_seeds,
            )?;
        }

        let seller_rec_acct = assert_is_ata(
            &seller_payment_receipt_account.to_account_info(),
            &seller.key(),
            &treasury_mint.key(),
        )?;

        // make sure you cant get rugged
        if seller_rec_acct.delegate.is_some() {
            return Err(AuctionHouseError::SellerATACannotHaveDelegate.into());
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &escrow_payment_account.key(),
                &seller_payment_receipt_account.key(),
                &auction_house.key(),
                &[],
                buyer_leftover_after_royalties_and_house_fee,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                token_program.to_account_info(),
                auction_house.to_account_info(),
            ],
            &[&ah_seeds],
        )?;
    } else {
        assert_keys_equal(seller_payment_receipt_account.key(), seller.key())?;
        invoke_signed(
            &system_instruction::transfer(
                &escrow_payment_account.key,
                seller_payment_receipt_account.key,
                buyer_leftover_after_royalties_and_house_fee,
            ),
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
    }

    if buyer_receipt_token_account.data_is_empty() {
        make_ata(
            buyer_receipt_token_account.to_account_info(),
            buyer.to_account_info(),
            token_mint.to_account_info(),
            fee_payer.to_account_info(),
            ata_program.to_account_info(),
            token_program.to_account_info(),
            system_program.to_account_info(),
            rent.to_account_info(),
            &fee_payer_seeds,
        )?;
    }

    let buyer_rec_acct = assert_is_ata(&buyer_receipt_clone, &buyer.key(), &token_mint.key())?;

    // make sure you cant get rugged
    if buyer_rec_acct.delegate.is_some() {
        return Err(AuctionHouseError::BuyerATACannotHaveDelegate.into());
    }

    let program_as_signer_seeds = [
        PREFIX.as_bytes(),
        SIGNER.as_bytes(),
        &[program_as_signer_bump],
    ];

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            &token_account.key(),
            &buyer_receipt_token_account.key(),
            &program_as_signer.key(),
            &[],
            token_size,
        )?,
        &[
            token_account.to_account_info(),
            buyer_receipt_clone,
            program_as_signer.to_account_info(),
            token_clone,
        ],
        &[&program_as_signer_seeds],
    )?;

    let curr_seller_lamp = seller_trade_state.lamports();
    **seller_trade_state.lamports.borrow_mut() = 0;
    sol_memset(&mut *seller_ts_data, 0, TRADE_STATE_SIZE);

    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_seller_lamp)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    let curr_buyer_lamp = buyer_trade_state.lamports();
    **buyer_trade_state.lamports.borrow_mut() = 0;
    sol_memset(&mut *buyer_ts_data, 0, TRADE_STATE_SIZE);
    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_buyer_lamp)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    if free_trade_state.lamports() > 0 {
        let curr_buyer_lamp = free_trade_state.lamports();
        **free_trade_state.lamports.borrow_mut() = 0;

        **fee_payer.lamports.borrow_mut() = fee_payer
            .lamports()
            .checked_add(curr_buyer_lamp)
            .ok_or(AuctionHouseError::NumericalOverflow)?;
        sol_memset(
            *free_trade_state.try_borrow_mut_data()?,
            0,
            TRADE_STATE_SIZE,
        );
    }
    Ok(())
}
