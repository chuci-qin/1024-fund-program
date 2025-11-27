//! Fund Program Processor
//!
//! Implements all instruction handlers for the Fund Program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::FundError,
    instruction::*,
    state::*,
    utils::*,
};

/// Process a Fund Program instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = FundInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        // Initialization
        FundInstruction::Initialize(args) => process_initialize(program_id, accounts, args),
        FundInstruction::CreateFund(args) => process_create_fund(program_id, accounts, args),
        
        // Fund Management
        FundInstruction::UpdateFund(args) => process_update_fund(program_id, accounts, args),
        FundInstruction::SetFundOpen(args) => process_set_fund_open(program_id, accounts, args),
        FundInstruction::SetFundPaused(args) => process_set_fund_paused(program_id, accounts, args),
        FundInstruction::CloseFund => process_close_fund(program_id, accounts),
        
        // LP Operations
        FundInstruction::DepositToFund(args) => process_deposit_to_fund(program_id, accounts, args),
        FundInstruction::RedeemFromFund(args) => process_redeem_from_fund(program_id, accounts, args),
        
        // Trading Operations
        FundInstruction::TradeFund(args) => process_trade_fund(program_id, accounts, args),
        FundInstruction::CloseFundPosition(args) => process_close_fund_position(program_id, accounts, args),
        
        // Fee Operations
        FundInstruction::CollectFees => process_collect_fees(program_id, accounts),
        
        // Admin Operations
        FundInstruction::UpdateAuthority(args) => process_update_authority(program_id, accounts, args),
        FundInstruction::SetProgramPaused(args) => process_set_program_paused(program_id, accounts, args),
        
        // NAV Operations
        FundInstruction::UpdateNAV => process_update_nav(program_id, accounts),
        FundInstruction::RecordPnL(args) => process_record_pnl(program_id, accounts, args),
        
        // Insurance Fund Operations
        FundInstruction::InitializeInsuranceFund(args) => process_initialize_insurance_fund(program_id, accounts, args),
        FundInstruction::AddLiquidationIncome(args) => process_add_liquidation_income(program_id, accounts, args),
        FundInstruction::AddADLProfit(args) => process_add_adl_profit(program_id, accounts, args),
        FundInstruction::CoverShortfall(args) => process_cover_shortfall(program_id, accounts, args),
        FundInstruction::UpdateHourlySnapshot => process_update_hourly_snapshot(program_id, accounts),
        FundInstruction::SetADLInProgress(args) => process_set_adl_in_progress(program_id, accounts, args),
        FundInstruction::CheckADLTrigger(args) => process_check_adl_trigger(program_id, accounts, args),
        FundInstruction::RedeemFromInsuranceFund(args) => process_redeem_from_insurance_fund(program_id, accounts, args),
        
        // Square Platform Operations
        FundInstruction::SquarePayment(args) => process_square_payment(program_id, accounts, args),
    }
}

// =============================================================================
// Initialization Instructions
// =============================================================================

/// Initialize the Fund Program configuration
fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let authority = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    // Verify authority is signer
    assert_signer(authority)?;
    
    // Derive FundConfig PDA
    let (config_pda, config_bump) = Pubkey::find_program_address(
        &[FUND_CONFIG_SEED],
        program_id,
    );
    
    if fund_config.key != &config_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Create FundConfig account
    let rent = Rent::get()?;
    let space = FundConfig::SIZE;
    let lamports = rent.minimum_balance(space);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            fund_config.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[authority.clone(), fund_config.clone(), system_program.clone()],
        &[&[FUND_CONFIG_SEED, &[config_bump]]],
    )?;
    
    // Initialize FundConfig
    let config = FundConfig::new(
        *authority.key,
        args.vault_program,
        args.ledger_program,
        config_bump,
    );
    
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Fund Program initialized");
    msg!("Authority: {}", authority.key);
    msg!("Vault Program: {}", args.vault_program);
    msg!("Ledger Program: {}", args.ledger_program);
    
    Ok(())
}

/// Create a new fund
fn process_create_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let share_mint = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let usdc_mint = next_account_info(account_info_iter)?;
    let _token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;
    
    // Verify manager is signer
    assert_signer(manager)?;
    
    // Validate fund name
    validate_fund_name(&args.name)?;
    
    // Validate fee configuration
    validate_fee_config(args.management_fee_bps, args.performance_fee_bps)?;
    
    // Load and update FundConfig
    let mut config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    if config.discriminator != FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::FundNotInitialized.into());
    }
    
    if config.is_paused {
        return Err(FundError::FundPaused.into());
    }
    
    let fund_index = config.total_funds;
    config.total_funds = config.total_funds.saturating_add(1);
    config.active_funds = config.active_funds.saturating_add(1);
    
    // Derive Fund PDA
    let fund_seeds = Fund::seeds(manager.key, fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    if fund_account.key != &fund_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Derive Fund vault PDA
    let vault_seeds = Fund::vault_seeds(&fund_pda);
    let vault_seeds_refs: Vec<&[u8]> = vault_seeds.iter().map(|s| s.as_slice()).collect();
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&vault_seeds_refs, program_id);
    
    if fund_vault.key != &vault_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Derive Share mint PDA
    let mint_seeds = Fund::share_mint_seeds(&fund_pda);
    let mint_seeds_refs: Vec<&[u8]> = mint_seeds.iter().map(|s| s.as_slice()).collect();
    let (mint_pda, mint_bump) = Pubkey::find_program_address(&mint_seeds_refs, program_id);
    
    if share_mint.key != &mint_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    let rent = Rent::get()?;
    let current_ts = get_current_timestamp()?;
    
    // Create Fund account
    let fund_space = Fund::SIZE;
    let fund_lamports = rent.minimum_balance(fund_space);
    
    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            fund_account.key,
            fund_lamports,
            fund_space as u64,
            program_id,
        ),
        &[manager.clone(), fund_account.clone(), system_program.clone()],
        &[&[FUND_SEED, manager.key.as_ref(), &fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Create Share mint (SPL Token)
    let mint_space = spl_token::state::Mint::LEN;
    let mint_lamports = rent.minimum_balance(mint_space);
    
    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            share_mint.key,
            mint_lamports,
            mint_space as u64,
            &spl_token::id(),
        ),
        &[manager.clone(), share_mint.clone(), system_program.clone()],
        &[&[SHARE_MINT_SEED, fund_pda.as_ref(), &[mint_bump]]],
    )?;
    
    // Initialize Share mint
    invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            share_mint.key,
            &fund_pda, // Mint authority = Fund PDA
            Some(&fund_pda), // Freeze authority = Fund PDA
            6, // 6 decimals like USDC
        )?,
        &[share_mint.clone(), rent_sysvar.clone()],
        &[&[SHARE_MINT_SEED, fund_pda.as_ref(), &[mint_bump]]],
    )?;
    
    // Create Fund vault (token account)
    let vault_space = spl_token::state::Account::LEN;
    let vault_lamports = rent.minimum_balance(vault_space);
    
    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            fund_vault.key,
            vault_lamports,
            vault_space as u64,
            &spl_token::id(),
        ),
        &[manager.clone(), fund_vault.clone(), system_program.clone()],
        &[&[FUND_VAULT_SEED, fund_pda.as_ref(), &[vault_bump]]],
    )?;
    
    // Initialize Fund vault
    invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            fund_vault.key,
            usdc_mint.key,
            &fund_pda, // Owner = Fund PDA
        )?,
        &[fund_vault.clone(), usdc_mint.clone(), fund_account.clone(), rent_sysvar.clone()],
        &[&[FUND_VAULT_SEED, fund_pda.as_ref(), &[vault_bump]]],
    )?;
    
    // Create fee config
    let fee_config = FeeConfig {
        management_fee_bps: args.management_fee_bps,
        performance_fee_bps: args.performance_fee_bps,
        use_high_water_mark: args.use_high_water_mark,
        fee_collection_interval: if args.fee_collection_interval > 0 {
            args.fee_collection_interval
        } else {
            FeeConfig::DEFAULT_COLLECTION_INTERVAL
        },
    };
    
    // Initialize Fund
    let fund = Fund::new(
        *manager.key,
        &args.name,
        fund_bump,
        *fund_vault.key,
        *share_mint.key,
        fee_config,
        fund_index,
        current_ts,
    );
    
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Fund created: {}", args.name);
    msg!("Fund index: {}", fund_index);
    msg!("Manager: {}", manager.key);
    msg!("Management fee: {} bps", args.management_fee_bps);
    msg!("Performance fee: {} bps", args.performance_fee_bps);
    
    Ok(())
}

// =============================================================================
// Fund Management Instructions
// =============================================================================

/// Update fund configuration
fn process_update_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: UpdateFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if fund.discriminator != FUND_DISCRIMINATOR {
        return Err(FundError::InvalidFundAccount.into());
    }
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    // Update fee config if provided
    if let Some(new_fee_config) = args.fee_config {
        validate_fee_config(new_fee_config.management_fee_bps, new_fee_config.performance_fee_bps)?;
        fund.fee_config = new_fee_config;
    }
    
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Fund updated: {}", fund.name_str());
    
    Ok(())
}

/// Set fund open/closed for deposits
fn process_set_fund_open(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetFundOpenArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    fund.is_open = args.is_open;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Fund {} is now {}", fund.name_str(), if args.is_open { "open" } else { "closed" });
    
    Ok(())
}

/// Set fund paused/unpaused
fn process_set_fund_paused(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetFundPausedArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    fund.is_paused = args.is_paused;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Fund {} is now {}", fund.name_str(), if args.is_paused { "paused" } else { "unpaused" });
    
    Ok(())
}

/// Close a fund
fn process_close_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let _share_mint = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let manager_usdc = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    // Check no LP positions remain
    if fund.stats.lp_count > 0 {
        return Err(FundError::FundHasLPPositions.into());
    }
    
    // Check no shares outstanding
    if fund.stats.total_shares > 0 {
        return Err(FundError::FundHasLPPositions.into());
    }
    
    // Transfer remaining funds to manager
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    if vault_account.amount > 0 {
        let fund_seeds = Fund::seeds(manager.key, fund.fund_index);
        let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
        let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
        
        invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                fund_vault.key,
                manager_usdc.key,
                fund_account.key,
                &[],
                vault_account.amount,
            )?,
            &[fund_vault.clone(), manager_usdc.clone(), fund_account.clone(), token_program.clone()],
            &[&[FUND_SEED, manager.key.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
        )?;
    }
    
    // Update FundConfig
    let mut config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    config.active_funds = config.active_funds.saturating_sub(1);
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Fund closed: {}", fund.name_str());
    
    Ok(())
}

// =============================================================================
// LP Operations
// =============================================================================

/// Deposit USDC into a fund
fn process_deposit_to_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: DepositToFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let investor = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let investor_usdc = next_account_info(account_info_iter)?;
    let lp_position = next_account_info(account_info_iter)?;
    let investor_shares = next_account_info(account_info_iter)?;
    let share_mint = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    assert_signer(investor)?;
    assert_owned_by(fund_account, program_id)?;
    
    if args.amount == 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    let amount_e6 = args.amount as i64;
    if amount_e6 < MIN_DEPOSIT_AMOUNT_E6 {
        return Err(FundError::DepositTooSmall.into());
    }
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if fund.discriminator != FUND_DISCRIMINATOR {
        return Err(FundError::InvalidFundAccount.into());
    }
    
    if !fund.can_deposit() {
        return Err(FundError::FundClosed.into());
    }
    
    let current_ts = get_current_timestamp()?;
    
    // Calculate shares to mint
    let shares = calculate_shares_to_mint(amount_e6, fund.stats.current_nav_e6)?;
    
    // Transfer USDC to fund vault
    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            investor_usdc.key,
            fund_vault.key,
            investor.key,
            &[],
            args.amount,
        )?,
        &[investor_usdc.clone(), fund_vault.clone(), investor.clone(), token_program.clone()],
    )?;
    
    // Mint share tokens to investor
    let fund_seeds = Fund::seeds(&fund.manager, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            share_mint.key,
            investor_shares.key,
            fund_account.key,
            &[],
            shares,
        )?,
        &[share_mint.clone(), investor_shares.clone(), fund_account.clone(), token_program.clone()],
        &[&[FUND_SEED, fund.manager.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Update or create LP position
    let lp_seeds = LPPosition::seeds(fund_account.key, investor.key);
    let lp_seeds_refs: Vec<&[u8]> = lp_seeds.iter().map(|s| s.as_slice()).collect();
    let (lp_pda, lp_bump) = Pubkey::find_program_address(&lp_seeds_refs, program_id);
    
    if lp_position.key != &lp_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    if lp_position.data_is_empty() {
        // Create new LP position
        let rent = Rent::get()?;
        let lp_space = LPPosition::SIZE;
        let lp_lamports = rent.minimum_balance(lp_space);
        
        invoke_signed(
            &system_instruction::create_account(
                investor.key,
                lp_position.key,
                lp_lamports,
                lp_space as u64,
                program_id,
            ),
            &[investor.clone(), lp_position.clone(), system_program.clone()],
            &[&[LP_POSITION_SEED, fund_account.key.as_ref(), investor.key.as_ref(), &[lp_bump]]],
        )?;
        
        let position = LPPosition::new(
            *fund_account.key,
            *investor.key,
            shares,
            fund.stats.current_nav_e6,
            amount_e6,
            current_ts,
            lp_bump,
        );
        position.serialize(&mut *lp_position.data.borrow_mut())?;
        
        // Increment LP count
        fund.stats.lp_count = fund.stats.lp_count.saturating_add(1);
    } else {
        // Update existing LP position
        let mut position = LPPosition::try_from_slice(&lp_position.data.borrow())?;
        position.add_shares(shares, amount_e6, fund.stats.current_nav_e6, current_ts)?;
        position.serialize(&mut *lp_position.data.borrow_mut())?;
    }
    
    // Update fund stats
    fund.record_deposit(amount_e6, shares)?;
    fund.last_update_ts = current_ts;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Deposit to fund: {} USDC", args.amount);
    msg!("Shares minted: {}", shares);
    msg!("Current NAV: {}", fund.stats.current_nav_e6);
    
    Ok(())
}

/// Redeem shares from a fund
fn process_redeem_from_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RedeemFromFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let investor = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let investor_usdc = next_account_info(account_info_iter)?;
    let lp_position = next_account_info(account_info_iter)?;
    let investor_shares = next_account_info(account_info_iter)?;
    let share_mint = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    assert_signer(investor)?;
    assert_owned_by(fund_account, program_id)?;
    
    if args.shares == 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.can_withdraw() {
        return Err(FundError::FundPaused.into());
    }
    
    let current_ts = get_current_timestamp()?;
    
    // Calculate redemption value
    let redemption_value = calculate_redemption_value(args.shares, fund.stats.current_nav_e6)?;
    
    // Check fund has enough balance
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    if vault_account.amount < redemption_value as u64 {
        return Err(FundError::InsufficientBalance.into());
    }
    
    // Update LP position
    let mut position = LPPosition::try_from_slice(&lp_position.data.borrow())?;
    
    if position.fund != *fund_account.key || position.investor != *investor.key {
        return Err(FundError::LPPositionNotFound.into());
    }
    
    if position.shares < args.shares {
        return Err(FundError::InsufficientShares.into());
    }
    
    position.remove_shares(args.shares, redemption_value, current_ts)?;
    
    // Burn share tokens
    invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            investor_shares.key,
            share_mint.key,
            investor.key,
            &[],
            args.shares,
        )?,
        &[investor_shares.clone(), share_mint.clone(), investor.clone(), token_program.clone()],
    )?;
    
    // Transfer USDC to investor
    let fund_seeds = Fund::seeds(&fund.manager, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            fund_vault.key,
            investor_usdc.key,
            fund_account.key,
            &[],
            redemption_value as u64,
        )?,
        &[fund_vault.clone(), investor_usdc.clone(), fund_account.clone(), token_program.clone()],
        &[&[FUND_SEED, fund.manager.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Check if position is empty
    if position.is_empty() {
        fund.stats.lp_count = fund.stats.lp_count.saturating_sub(1);
    }
    
    position.serialize(&mut *lp_position.data.borrow_mut())?;
    
    // Update fund stats
    fund.record_withdrawal(redemption_value, args.shares)?;
    fund.last_update_ts = current_ts;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Redeem from fund: {} shares", args.shares);
    msg!("USDC received: {}", redemption_value);
    msg!("Current NAV: {}", fund.stats.current_nav_e6);
    
    Ok(())
}

// =============================================================================
// Trading Operations
// =============================================================================

/// Trade using fund assets
fn process_trade_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TradeFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let ledger_program = next_account_info(account_info_iter)?;
    let position = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let vault_config = next_account_info(account_info_iter)?;
    let ledger_config = next_account_info(account_info_iter)?;
    let user_stats = next_account_info(account_info_iter)?;
    let vault_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    if fund.is_paused {
        return Err(FundError::FundPaused.into());
    }
    
    // Verify Ledger Program
    let config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    if ledger_program.key != &config.ledger_program {
        return Err(FundError::InvalidAccountOwner.into());
    }
    
    // CPI call to Ledger Program to open position
    let fund_seeds = Fund::seeds(manager.key, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    // Generate batch ID from timestamp
    let batch_id = get_current_timestamp()? as u64;
    
    crate::cpi::open_position(
        ledger_program.key,
        fund_account.clone(),  // Fund acts as relayer
        position.clone(),
        user_account.clone(),
        vault_config.clone(),
        ledger_config.clone(),
        user_stats.clone(),
        vault_program.clone(),
        system_program.clone(),
        *fund_account.key,  // User is the fund itself
        args.market_index,
        args.side,
        args.size_e6,
        args.price_e6,
        args.leverage,
        batch_id,
        &[&[FUND_SEED, manager.key.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    msg!("Trade fund: market={}, side={}, size={}, leverage={}, batch_id={}",
        args.market_index, args.side, args.size_e6, args.leverage, batch_id);
    
    Ok(())
}

/// Close a fund position
fn process_close_fund_position(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CloseFundPositionArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let ledger_program = next_account_info(account_info_iter)?;
    let position = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let vault_config = next_account_info(account_info_iter)?;
    let insurance_fund = next_account_info(account_info_iter)?;
    let ledger_config = next_account_info(account_info_iter)?;
    let user_stats = next_account_info(account_info_iter)?;
    let vault_program = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    // Verify Ledger Program
    let config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    if ledger_program.key != &config.ledger_program {
        return Err(FundError::InvalidAccountOwner.into());
    }
    
    // CPI call to Ledger Program to close position
    let fund_seeds = Fund::seeds(manager.key, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    // Generate batch ID from timestamp
    let batch_id = get_current_timestamp()? as u64;
    
    crate::cpi::close_position(
        ledger_program.key,
        fund_account.clone(),  // Fund acts as relayer
        position.clone(),
        user_account.clone(),
        vault_config.clone(),
        insurance_fund.clone(),
        ledger_config.clone(),
        user_stats.clone(),
        vault_program.clone(),
        *fund_account.key,  // User is the fund itself
        args.market_index,
        args.size_e6,
        args.price_e6,
        batch_id,
        &[&[FUND_SEED, manager.key.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    msg!("Close fund position: market={}, size={}, price={}, batch_id={}",
        args.market_index, args.size_e6, args.price_e6, batch_id);
    
    Ok(())
}

// =============================================================================
// Fee Operations
// =============================================================================

/// Collect management and performance fees
fn process_collect_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let manager = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let manager_usdc = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    assert_signer(manager)?;
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    if !fund.is_manager(manager.key) {
        return Err(FundError::NotFundManager.into());
    }
    
    let current_ts = get_current_timestamp()?;
    
    // Check fee collection interval
    if !can_collect_fees(fund.stats.last_fee_collection_ts, fund.fee_config.fee_collection_interval)? {
        return Err(FundError::FeeCollectionTooEarly.into());
    }
    
    // Calculate fees
    let (mgmt_fee, perf_fee) = fund.calculate_fees(current_ts)?;
    let total_fee = safe_add_i64(mgmt_fee, perf_fee)?;
    
    if total_fee <= 0 {
        return Err(FundError::NoFeesToCollect.into());
    }
    
    // Transfer fees to manager
    let fund_seeds = Fund::seeds(manager.key, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            fund_vault.key,
            manager_usdc.key,
            fund_account.key,
            &[],
            total_fee as u64,
        )?,
        &[fund_vault.clone(), manager_usdc.clone(), fund_account.clone(), token_program.clone()],
        &[&[FUND_SEED, manager.key.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Update fund state
    fund.collect_fees(mgmt_fee, perf_fee, current_ts)?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Fees collected:");
    msg!("  Management fee: {}", mgmt_fee);
    msg!("  Performance fee: {}", perf_fee);
    msg!("  Total: {}", total_fee);
    
    Ok(())
}

// =============================================================================
// Admin Operations
// =============================================================================

/// Update program authority
fn process_update_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: UpdateAuthorityArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let authority = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    
    assert_signer(authority)?;
    assert_owned_by(fund_config, program_id)?;
    
    let mut config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    
    if config.authority != *authority.key {
        return Err(FundError::AdminRequired.into());
    }
    
    config.authority = args.new_authority;
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Authority updated to: {}", args.new_authority);
    
    Ok(())
}

/// Set program paused state
fn process_set_program_paused(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetProgramPausedArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let authority = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    
    assert_signer(authority)?;
    assert_owned_by(fund_config, program_id)?;
    
    let mut config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    
    if config.authority != *authority.key {
        return Err(FundError::AdminRequired.into());
    }
    
    config.is_paused = args.is_paused;
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Program is now {}", if args.is_paused { "paused" } else { "unpaused" });
    
    Ok(())
}

// =============================================================================
// NAV Operations
// =============================================================================

/// Update NAV for a fund
fn process_update_nav(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let fund_account = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    fund.stats.update_nav()?;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("NAV updated: {}", fund.stats.current_nav_e6);
    
    Ok(())
}

/// Record realized PnL (CPI from Ledger)
fn process_record_pnl(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RecordPnLArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    
    // Verify caller is Ledger Program
    let config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    if config.discriminator != FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::FundNotInitialized.into());
    }
    
    // Verify the caller is the authorized Ledger Program
    if caller.key != &config.ledger_program {
        msg!("Unauthorized caller: expected {}, got {}", config.ledger_program, caller.key);
        return Err(FundError::UnauthorizedCaller.into());
    }
    
    assert_owned_by(fund_account, program_id)?;
    
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    fund.record_pnl(args.pnl_e6)?;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("PnL recorded: {}", args.pnl_e6);
    msg!("New NAV: {}", fund.stats.current_nav_e6);
    
    Ok(())
}

// =============================================================================
// Insurance Fund Operations
// =============================================================================

/// Initialize the Insurance Fund
/// 
/// Creates a special Fund instance for the Insurance Fund along with its
/// InsuranceFundConfig account.
fn process_initialize_insurance_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeInsuranceFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let authority = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let share_mint = next_account_info(account_info_iter)?;
    let fund_config = next_account_info(account_info_iter)?;
    let usdc_mint = next_account_info(account_info_iter)?;
    let _token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;
    
    // Verify authority is signer
    assert_signer(authority)?;
    
    // Load FundConfig and verify authority
    let mut config = FundConfig::try_from_slice(&fund_config.data.borrow())?;
    if config.discriminator != FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::FundNotInitialized.into());
    }
    if config.authority != *authority.key {
        return Err(FundError::AdminRequired.into());
    }
    if config.is_paused {
        return Err(FundError::FundPaused.into());
    }
    
    let fund_index = config.total_funds;
    let current_ts = get_current_timestamp()?;
    let rent = Rent::get()?;
    
    // Derive InsuranceFundConfig PDA
    let (insurance_config_pda, insurance_config_bump) = Pubkey::find_program_address(
        &[INSURANCE_FUND_CONFIG_SEED],
        program_id,
    );
    
    if insurance_config.key != &insurance_config_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Check if already initialized
    if !insurance_config.data_is_empty() {
        return Err(FundError::InsuranceFundAlreadyInitialized.into());
    }
    
    // Derive Fund PDA for insurance fund (use authority as manager, special index)
    let fund_seeds = Fund::seeds(authority.key, fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    if fund_account.key != &fund_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Derive vault and mint PDAs
    let vault_seeds = Fund::vault_seeds(&fund_pda);
    let vault_seeds_refs: Vec<&[u8]> = vault_seeds.iter().map(|s| s.as_slice()).collect();
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&vault_seeds_refs, program_id);
    
    if fund_vault.key != &vault_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    let mint_seeds = Fund::share_mint_seeds(&fund_pda);
    let mint_seeds_refs: Vec<&[u8]> = mint_seeds.iter().map(|s| s.as_slice()).collect();
    let (mint_pda, mint_bump) = Pubkey::find_program_address(&mint_seeds_refs, program_id);
    
    if share_mint.key != &mint_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Create Fund account
    let fund_space = Fund::SIZE;
    let fund_lamports = rent.minimum_balance(fund_space);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            fund_account.key,
            fund_lamports,
            fund_space as u64,
            program_id,
        ),
        &[authority.clone(), fund_account.clone(), system_program.clone()],
        &[&[FUND_SEED, authority.key.as_ref(), &fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Create Share mint (SPL Token)
    let mint_space = spl_token::state::Mint::LEN;
    let mint_lamports = rent.minimum_balance(mint_space);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            share_mint.key,
            mint_lamports,
            mint_space as u64,
            &spl_token::id(),
        ),
        &[authority.clone(), share_mint.clone(), system_program.clone()],
        &[&[SHARE_MINT_SEED, fund_pda.as_ref(), &[mint_bump]]],
    )?;
    
    // Initialize Share mint
    invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            share_mint.key,
            &fund_pda,
            Some(&fund_pda),
            6,
        )?,
        &[share_mint.clone(), rent_sysvar.clone()],
        &[&[SHARE_MINT_SEED, fund_pda.as_ref(), &[mint_bump]]],
    )?;
    
    // Create Fund vault (token account)
    let vault_space = spl_token::state::Account::LEN;
    let vault_lamports = rent.minimum_balance(vault_space);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            fund_vault.key,
            vault_lamports,
            vault_space as u64,
            &spl_token::id(),
        ),
        &[authority.clone(), fund_vault.clone(), system_program.clone()],
        &[&[FUND_VAULT_SEED, fund_pda.as_ref(), &[vault_bump]]],
    )?;
    
    // Initialize Fund vault
    invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            fund_vault.key,
            usdc_mint.key,
            &fund_pda,
        )?,
        &[fund_vault.clone(), usdc_mint.clone(), fund_account.clone(), rent_sysvar.clone()],
        &[&[FUND_VAULT_SEED, fund_pda.as_ref(), &[vault_bump]]],
    )?;
    
    // Create InsuranceFundConfig account
    let insurance_config_space = InsuranceFundConfig::SIZE;
    let insurance_config_lamports = rent.minimum_balance(insurance_config_space);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            insurance_config.key,
            insurance_config_lamports,
            insurance_config_space as u64,
            program_id,
        ),
        &[authority.clone(), insurance_config.clone(), system_program.clone()],
        &[&[INSURANCE_FUND_CONFIG_SEED, &[insurance_config_bump]]],
    )?;
    
    // Initialize Fund (no management/performance fees for insurance fund)
    let fee_config = FeeConfig {
        management_fee_bps: 0,
        performance_fee_bps: 0,
        use_high_water_mark: false,
        fee_collection_interval: 0,
    };
    
    let fund = Fund::new(
        *authority.key,
        "1024 Insurance Fund",
        fund_bump,
        *fund_vault.key,
        *share_mint.key,
        fee_config,
        fund_index,
        current_ts,
    );
    
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    // Initialize InsuranceFundConfig
    let insurance_fund_config = InsuranceFundConfig::new(
        *fund_account.key,
        insurance_config_bump,
        args.adl_trigger_threshold_e6,
        args.withdrawal_delay_secs,
        args.authorized_caller,
        current_ts,
    );
    
    insurance_fund_config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    // Update FundConfig
    config.total_funds = config.total_funds.saturating_add(1);
    config.active_funds = config.active_funds.saturating_add(1);
    config.serialize(&mut *fund_config.data.borrow_mut())?;
    
    msg!("Insurance Fund initialized");
    msg!("Fund: {}", fund_account.key);
    msg!("Config: {}", insurance_config.key);
    msg!("ADL threshold: {}", args.adl_trigger_threshold_e6);
    msg!("Withdrawal delay: {} seconds", args.withdrawal_delay_secs);
    
    Ok(())
}

/// Add liquidation income to Insurance Fund (CPI from Ledger)
fn process_add_liquidation_income(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddLiquidationIncomeArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    // Load and verify InsuranceFundConfig
    let mut config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // Verify caller is authorized
    if !config.is_authorized_caller(caller.key) {
        msg!("Unauthorized caller: {}", caller.key);
        return Err(FundError::UnauthorizedCaller.into());
    }
    
    // Update stats
    config.add_liquidation_income(args.amount_e6);
    config.last_update_ts = get_current_timestamp()?;
    config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    // Update Fund's realized PnL (income is positive PnL for the fund)
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    fund.record_pnl(args.amount_e6)?;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("Liquidation income added: {}", args.amount_e6);
    msg!("Total liquidation income: {}", config.total_liquidation_income_e6);
    
    Ok(())
}

/// Add ADL profit to Insurance Fund (CPI from Ledger)
fn process_add_adl_profit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddADLProfitArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    // Load and verify InsuranceFundConfig
    let mut config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // Verify caller is authorized
    if !config.is_authorized_caller(caller.key) {
        msg!("Unauthorized caller: {}", caller.key);
        return Err(FundError::UnauthorizedCaller.into());
    }
    
    // Update stats
    config.add_adl_profit(args.amount_e6);
    config.last_update_ts = get_current_timestamp()?;
    config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    // Update Fund's realized PnL
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    fund.record_pnl(args.amount_e6)?;
    fund.last_update_ts = get_current_timestamp()?;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!("ADL profit added: {}", args.amount_e6);
    msg!("Total ADL profit: {}", config.total_adl_profit_e6);
    
    Ok(())
}

/// Cover shortfall from Insurance Fund (CPI from Ledger)
fn process_cover_shortfall(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CoverShortfallArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let destination = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    // Load and verify InsuranceFundConfig
    let mut config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // Verify caller is authorized
    if !config.is_authorized_caller(caller.key) {
        msg!("Unauthorized caller: {}", caller.key);
        return Err(FundError::UnauthorizedCaller.into());
    }
    
    // Get current balance
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    let current_balance = vault_account.amount as i64;
    
    // Calculate coverage
    let (covered, remaining) = config.cover_shortfall(args.shortfall_e6, current_balance);
    
    if covered > 0 {
        // Transfer covered amount from insurance fund
        let fund = Fund::try_from_slice(&fund_account.data.borrow())?;
        let fund_seeds = Fund::seeds(&fund.manager, fund.fund_index);
        let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
        let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
        
        invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                fund_vault.key,
                destination.key,
                fund_account.key,
                &[],
                covered as u64,
            )?,
            &[fund_vault.clone(), destination.clone(), fund_account.clone(), token_program.clone()],
            &[&[FUND_SEED, fund.manager.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
        )?;
        
        // Update Fund stats (shortfall is negative PnL)
        let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
        fund.record_pnl(-covered)?;
        fund.last_update_ts = get_current_timestamp()?;
        fund.serialize(&mut *fund_account.data.borrow_mut())?;
    }
    
    config.last_update_ts = get_current_timestamp()?;
    config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    msg!("Shortfall coverage:");
    msg!("  Requested: {}", args.shortfall_e6);
    msg!("  Covered: {}", covered);
    msg!("  Remaining (needs ADL): {}", remaining);
    
    if remaining > 0 {
        msg!("⚠️ Insurance Fund insufficient, ADL required for: {}", remaining);
    }
    
    Ok(())
}

/// Update hourly snapshot (for 30% decline trigger condition)
fn process_update_hourly_snapshot(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let _caller = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    // Load InsuranceFundConfig
    let mut config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    let current_ts = get_current_timestamp()?;
    
    // Check minimum 1 hour between snapshots
    let one_hour: i64 = 3600;
    if current_ts - config.last_snapshot_ts < one_hour {
        msg!("Snapshot too recent, last: {}, now: {}", config.last_snapshot_ts, current_ts);
        return Err(FundError::SnapshotTooRecent.into());
    }
    
    // Get current balance
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    let current_balance = vault_account.amount as i64;
    
    // Update snapshot
    config.update_hourly_snapshot(current_balance, current_ts);
    config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    msg!("Hourly snapshot updated");
    msg!("  Balance: {}", current_balance);
    msg!("  Timestamp: {}", current_ts);
    
    Ok(())
}

/// Set ADL in progress status (CPI from Ledger)
fn process_set_adl_in_progress(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetADLInProgressArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    
    assert_owned_by(insurance_config, program_id)?;
    
    // Load and verify InsuranceFundConfig
    let mut config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // Verify caller is authorized
    if !config.is_authorized_caller(caller.key) {
        msg!("Unauthorized caller: {}", caller.key);
        return Err(FundError::UnauthorizedCaller.into());
    }
    
    config.set_adl_in_progress(args.in_progress);
    config.last_update_ts = get_current_timestamp()?;
    config.serialize(&mut *insurance_config.data.borrow_mut())?;
    
    msg!("ADL in progress: {}", args.in_progress);
    if args.in_progress {
        msg!("⚠️ LP redemptions are now paused");
    } else {
        msg!("✅ LP redemptions resumed");
    }
    
    Ok(())
}

/// Check ADL trigger conditions (view function)
fn process_check_adl_trigger(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CheckADLTriggerArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    // Load InsuranceFundConfig
    let config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // Get current balance
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    let current_balance = vault_account.amount as i64;
    
    // Check trigger conditions
    let trigger_reason = config.should_trigger_adl(current_balance, args.shortfall_e6);
    
    msg!("ADL Trigger Check:");
    msg!("  Current balance: {}", current_balance);
    msg!("  1h ago balance: {}", config.balance_1h_ago_e6);
    msg!("  ADL threshold: {}", config.adl_trigger_threshold_e6);
    msg!("  Shortfall: {}", args.shortfall_e6);
    
    match trigger_reason {
        ADLTriggerReason::None => {
            msg!("  Result: ✅ No ADL required");
        }
        ADLTriggerReason::Bankruptcy => {
            msg!("  Result: ⚠️ BANKRUPTCY - Insurance fund cannot cover shortfall");
        }
        ADLTriggerReason::InsufficientBalance => {
            msg!("  Result: ⚠️ INSUFFICIENT BALANCE - Below ADL threshold");
        }
        ADLTriggerReason::RapidDecline => {
            msg!("  Result: ⚠️ RAPID DECLINE - Balance dropped >30% in 1 hour");
        }
    }
    
    Ok(())
}

/// Redeem shares from Insurance Fund (with special rules)
/// 
/// Special rules:
/// 1. ADL in progress: redemption is paused
/// 2. Withdrawal delay: must wait for configured delay
fn process_redeem_from_insurance_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RedeemFromInsuranceFundArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let investor = next_account_info(account_info_iter)?;
    let fund_account = next_account_info(account_info_iter)?;
    let insurance_config = next_account_info(account_info_iter)?;
    let fund_vault = next_account_info(account_info_iter)?;
    let investor_usdc = next_account_info(account_info_iter)?;
    let lp_position = next_account_info(account_info_iter)?;
    let investor_shares = next_account_info(account_info_iter)?;
    let share_mint = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    
    assert_signer(investor)?;
    assert_owned_by(fund_account, program_id)?;
    assert_owned_by(insurance_config, program_id)?;
    
    if args.shares == 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    // Load InsuranceFundConfig
    let config = InsuranceFundConfig::try_from_slice(&insurance_config.data.borrow())?;
    if config.discriminator != INSURANCE_FUND_CONFIG_DISCRIMINATOR {
        return Err(FundError::InsuranceFundNotInitialized.into());
    }
    
    // === Special Rule 1: Check ADL in progress ===
    if config.is_adl_in_progress {
        msg!("❌ Insurance Fund redemption paused: ADL in progress");
        return Err(FundError::ADLInProgress.into());
    }
    
    // Load Fund
    let mut fund = Fund::try_from_slice(&fund_account.data.borrow())?;
    
    // Verify this is the Insurance Fund
    if fund.fund_vault != *fund_vault.key || config.fund != *fund_account.key {
        return Err(FundError::InvalidFundAccount.into());
    }
    
    if !fund.can_withdraw() {
        return Err(FundError::FundPaused.into());
    }
    
    let current_ts = get_current_timestamp()?;
    
    // Load LP position
    let mut position = LPPosition::try_from_slice(&lp_position.data.borrow())?;
    
    if position.fund != *fund_account.key || position.investor != *investor.key {
        return Err(FundError::LPPositionNotFound.into());
    }
    
    if position.shares < args.shares {
        return Err(FundError::InsufficientShares.into());
    }
    
    // === Special Rule 2: Check withdrawal delay ===
    // For Insurance Fund, there's a delay between request and execution
    // For simplicity, we check against last_update_ts as the "request time"
    if config.withdrawal_delay_secs > 0 {
        let time_since_last_update = current_ts - position.last_update_ts;
        if time_since_last_update < config.withdrawal_delay_secs {
            let remaining = config.withdrawal_delay_secs - time_since_last_update;
            msg!(
                "❌ Insurance Fund redemption delayed: {} seconds remaining",
                remaining
            );
            return Err(FundError::WithdrawalDelayNotMet.into());
        }
    }
    
    // Calculate redemption value
    let redemption_value = calculate_redemption_value(args.shares, fund.stats.current_nav_e6)?;
    
    // Check fund has enough balance
    let vault_account = spl_token::state::Account::unpack(&fund_vault.data.borrow())?;
    if vault_account.amount < redemption_value as u64 {
        return Err(FundError::InsufficientBalance.into());
    }
    
    // Update LP position
    position.remove_shares(args.shares, redemption_value, current_ts)?;
    
    // Burn share tokens
    invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            investor_shares.key,
            share_mint.key,
            investor.key,
            &[],
            args.shares,
        )?,
        &[investor_shares.clone(), share_mint.clone(), investor.clone(), token_program.clone()],
    )?;
    
    // Transfer USDC to investor
    let fund_seeds = Fund::seeds(&fund.manager, fund.fund_index);
    let fund_seeds_refs: Vec<&[u8]> = fund_seeds.iter().map(|s| s.as_slice()).collect();
    let (_, fund_bump) = Pubkey::find_program_address(&fund_seeds_refs, program_id);
    
    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            fund_vault.key,
            investor_usdc.key,
            fund_account.key,
            &[],
            redemption_value as u64,
        )?,
        &[fund_vault.clone(), investor_usdc.clone(), fund_account.clone(), token_program.clone()],
        &[&[FUND_SEED, fund.manager.as_ref(), &fund.fund_index.to_le_bytes(), &[fund_bump]]],
    )?;
    
    // Check if position is empty
    if position.is_empty() {
        fund.stats.lp_count = fund.stats.lp_count.saturating_sub(1);
    }
    
    position.serialize(&mut *lp_position.data.borrow_mut())?;
    
    // Update fund stats
    fund.record_withdrawal(redemption_value, args.shares)?;
    fund.last_update_ts = current_ts;
    fund.serialize(&mut *fund_account.data.borrow_mut())?;
    
    msg!(
        "✅ Insurance Fund redemption: {} shares = {} lamports",
        args.shares,
        redemption_value
    );
    
    Ok(())
}

// =============================================================================
// Square Platform Operations
// =============================================================================

/// Process a Square platform payment
/// 
/// Records payment on-chain, transfers creator share to their account,
/// and platform share to Square Fund.
fn process_square_payment(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SquarePaymentArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let payer = next_account_info(account_info_iter)?;
    let payment_record = next_account_info(account_info_iter)?;
    let payer_vault = next_account_info(account_info_iter)?;
    let creator_vault = next_account_info(account_info_iter)?;
    let square_fund_vault = next_account_info(account_info_iter)?;
    let _vault_program = next_account_info(account_info_iter)?; // Reserved for future CPI
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    
    // Verify payer is signer
    assert_signer(payer)?;
    
    if args.amount_e6 <= 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    if args.creator_share_bps > 10000 {
        return Err(FundError::InvalidFeeConfiguration.into());
    }
    
    let current_ts = get_current_timestamp()?;
    let rent = Rent::get()?;
    
    // Convert payment type
    let payment_type = match args.payment_type {
        0 => SquarePaymentType::KnowledgePurchase,
        1 => SquarePaymentType::Subscription,
        2 => SquarePaymentType::LiveDonation,
        _ => return Err(FundError::InvalidPaymentType.into()),
    };
    
    // Derive SquarePaymentRecord PDA
    let record_seeds = SquarePaymentRecord::seeds(payer.key, args.content_id, current_ts);
    let record_seeds_refs: Vec<&[u8]> = record_seeds.iter().map(|s| s.as_slice()).collect();
    let (record_pda, record_bump) = Pubkey::find_program_address(&record_seeds_refs, program_id);
    
    if payment_record.key != &record_pda {
        return Err(FundError::InvalidPDA.into());
    }
    
    // Check record doesn't already exist
    if !payment_record.data_is_empty() {
        return Err(FundError::PaymentRecordAlreadyExists.into());
    }
    
    // Calculate amounts
    let creator_amount_e6 = (args.amount_e6 as i128 * args.creator_share_bps as i128 / 10000) as i64;
    let platform_amount_e6 = args.amount_e6.saturating_sub(creator_amount_e6);
    
    // Create payment record account
    let record_space = SquarePaymentRecord::SIZE;
    let record_lamports = rent.minimum_balance(record_space);
    
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            payment_record.key,
            record_lamports,
            record_space as u64,
            program_id,
        ),
        &[payer.clone(), payment_record.clone(), system_program.clone()],
        &[&[
            SQUARE_PAYMENT_RECORD_SEED,
            payer.key.as_ref(),
            &args.content_id.to_le_bytes(),
            &current_ts.to_le_bytes(),
            &[record_bump],
        ]],
    )?;
    
    // Initialize payment record
    let record = SquarePaymentRecord::new(
        *payer.key,
        args.creator,
        args.content_id,
        payment_type,
        args.amount_e6,
        args.creator_share_bps,
        current_ts,
        args.subscription_period,
        &args.memo,
        record_bump,
    );
    
    record.serialize(&mut *payment_record.data.borrow_mut())?;
    
    // Transfer creator share from payer vault to creator vault
    if creator_amount_e6 > 0 {
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                payer_vault.key,
                creator_vault.key,
                payer.key,
                &[],
                creator_amount_e6 as u64,
            )?,
            &[
                payer_vault.clone(),
                creator_vault.clone(),
                payer.clone(),
                token_program.clone(),
            ],
        )?;
    }
    
    // Transfer platform share from payer vault to square fund vault
    if platform_amount_e6 > 0 {
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                payer_vault.key,
                square_fund_vault.key,
                payer.key,
                &[],
                platform_amount_e6 as u64,
            )?,
            &[
                payer_vault.clone(),
                square_fund_vault.clone(),
                payer.clone(),
                token_program.clone(),
            ],
        )?;
    }
    
    msg!("📝 SQUARE_PAYMENT_RECORD:");
    msg!("  payer: {}", payer.key);
    msg!("  creator: {}", args.creator);
    msg!("  content_id: {}", args.content_id);
    msg!("  payment_type: {:?}", payment_type);
    msg!("  total_amount_e6: {}", args.amount_e6);
    msg!("  creator_amount_e6: {}", creator_amount_e6);
    msg!("  platform_amount_e6: {}", platform_amount_e6);
    msg!("  creator_share_bps: {}", args.creator_share_bps);
    msg!("  timestamp: {}", current_ts);
    msg!("  record: {}", payment_record.key);
    
    Ok(())
}
