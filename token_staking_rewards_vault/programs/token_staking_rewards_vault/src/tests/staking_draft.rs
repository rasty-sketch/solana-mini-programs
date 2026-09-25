use std::vec;

use anchor_lang::{prelude::pubkey, ToAccountMetas};
use anchor_spl::token_interface::TokenInterface;
use mollusk_svm::{program::keyed_account_for_system_program, result, Mollusk};
use mollusk_svm_programs_token::{associated_token, token};
use sha2::{Digest, Sha256};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

const ID: Pubkey = solana_pubkey::pubkey!("52hdHGSANrycFt1ku1qy4GWBE9cdKTRyaexTsu17SQ36");

fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{ix_name}"));
    let hash = hasher.finalize();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}

fn setup_vault_env() -> Mollusk {
    let sbf_out_dir = format!("{}/../../target/deploy", env!("CARGO_MANIFEST_DIR"));
    std::env::set_var("SBF_OUT_DIR", sbf_out_dir);

    let mut mollusk = Mollusk::new(&ID, "token_staking_rewards_vault");

    token::add_program(&mut mollusk);
    associated_token::add_program(&mut mollusk);
    mollusk
}

fn mint_maker(mint_authority: Pubkey) -> (Pubkey, Account) {
    let mint_address = Pubkey::new_unique();
    let mint_state = Mint {
        mint_authority: COption::Some(mint_authority),
        supply: 1_000_000_000,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mint_account = token::create_account_for_mint(mint_state);
    (mint_address, mint_account)
}

pub struct TestCtx {
    mollusk: Mollusk,
    admin: Pubkey,
    vault: Pubkey,
    staked_mint: Pubkey,
    reward_mint: Pubkey,
    staked_mint_ata: Pubkey,
    reward_mint_ata: Pubkey,
    reward_rate: u64,
}

fn user_generator(
    mint: Pubkey,
    vault: Pubkey,
    amount: u64,
) -> (Pubkey, Account, Pubkey, TokenAccount) {
    let user = Pubkey::new_unique();
    let user_account = Account::new(1_000_000_000, 0, &solana_sdk_ids::system_program::id());
    let (user_stake_profile, _bump) =
        Pubkey::find_program_address(&[b"vault", user.as_ref(), vault.as_ref()], &ID);

    let user_token_state = TokenAccount {
        mint: mint,
        owner: user,
        amount: amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    (user, user_account, user_stake_profile, user_token_state)
}

fn setup_vault_init(reward_rate: u64) -> (TestCtx, result::InstructionResult) {
    let mut mollusk = setup_vault_env();

    let admin = Pubkey::new_unique();
    let admin_account = Account::new(10_000_000_000, 0, &solana_sdk_ids::system_program::id());
    let (staked_mint, staked_mint_account) = mint_maker(admin);
    let (reward_mint, reward_mint_account) = mint_maker(admin);

    let (vault, _vault_bump) = Pubkey::find_program_address(&[b"vault", admin.as_ref()], &ID);

    let staked_mint_ata =
        get_associated_token_address_with_program_id(&vault, &staked_mint, &token::ID);

    let reward_mint_ata =
        get_associated_token_address_with_program_id(&vault, &reward_mint, &token::ID);

    let empty_system_account = Account::new(0, 0, &solana_sdk_ids::system_program::id());

    // let reward_rate: u64 = 5;
    let mut data = anchor_discriminator("vault_init").to_vec();
    data.extend_from_slice(&reward_rate.to_le_bytes());

    let ix = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new_readonly(staked_mint, false),
            AccountMeta::new_readonly(reward_mint, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(staked_mint_ata, false),
            AccountMeta::new(reward_mint_ata, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(associated_token::ID, false),
        ],
        data,
    };

    let input_accounts = vec![
        (admin, admin_account.clone()),
        (staked_mint, staked_mint_account.clone()),
        (reward_mint, reward_mint_account.clone()),
        (vault, empty_system_account.clone()),
        (staked_mint_ata, empty_system_account.clone()),
        (reward_mint_ata, empty_system_account.clone()),
        keyed_account_for_system_program(),
        token::keyed_account(),
        associated_token::keyed_account(),
    ];

    let result = mollusk.process_instruction(&ix, &input_accounts);

    (
        TestCtx {
            mollusk,
            admin,
            vault,
            staked_mint,
            reward_mint,
            staked_mint_ata,
            reward_mint_ata,
            reward_rate,
        },  
        result,
    )
}

#[test]
fn invariant_init_tests() {
    let (account_addresses, result) = setup_vault_init(5);

    assert!(
        result.program_result.is_ok(),
        "vault_init failed: {:?}",
        result.program_result
    );

    let vault_after = result
        .get_account(&account_addresses.vault)
        .expect("vault account is missing");
    assert_eq!(vault_after.owner, ID);  

    let data = &vault_after.data;
    assert!(data.len() >= 8 + 32 + 32 + 32 + 8 + 8 + 8 + 16 + 1);

    let body = &data[8..];

    let mut admin_bytes = [0_u8; 32];
    admin_bytes.copy_from_slice(&body[0..32]);
    let mut staked_mint_bytes = [0u8; 32];
    staked_mint_bytes.copy_from_slice(&body[32..64]);
    let mut reward_mint_bytes = [0u8; 32];
    reward_mint_bytes.copy_from_slice(&body[64..96]);

    let total_staked = u64::from_le_bytes(body[96..104].try_into().unwrap());
    let last_updated = i64::from_le_bytes(body[104..112].try_into().unwrap());
    let reward_rate_stored = u64::from_le_bytes(body[112..120].try_into().unwrap());
    let reward_per_token_stored = u128::from_le_bytes(body[120..136].try_into().unwrap());

    assert_eq!(Pubkey::new_from_array(admin_bytes), account_addresses.admin);
    assert_eq!(
        Pubkey::new_from_array(staked_mint_bytes),
        account_addresses.staked_mint
    );
    assert_eq!(
        Pubkey::new_from_array(reward_mint_bytes),
        account_addresses.reward_mint
    );

    assert!(total_staked == 0);
    assert!(last_updated == 0);
    assert!(reward_rate_stored == account_addresses.reward_rate);
    assert!(reward_per_token_stored == 0);

    let reward_ata_after = result
        .get_account(&account_addresses.reward_mint_ata)
        .expect("rewardmint ata is missing");
    let staked_ata_after = result
        .get_account(&account_addresses.staked_mint_ata)
        .expect("staked ata is missing");

    assert_eq!(staked_ata_after.owner, token::ID);
    assert_eq!(reward_ata_after.owner, token::ID);

    let _ = spl_token_interface::state::Account::unpack(&staked_ata_after.data).unwrap();
    let _ = spl_token_interface::state::Account::unpack(&reward_ata_after.data).unwrap();
}

// invariant test for deposit
#[test]
fn invariant_test_for_deposit() {
    let (account_addresses, result) = setup_vault_init(5);

    let vault_after = result
        .get_account(&account_addresses.vault)
        .expect("vault account is missing");

    let reward_ata_after = result
        .get_account(&account_addresses.reward_mint_ata)
        .expect("rewardmint ata is missing");
    let staked_ata_after = result
        .get_account(&account_addresses.staked_mint_ata)
        .expect("staked ata is missing");

    let (user, user_account, user_stake_profile, user_staked_token_state) =
        user_generator(account_addresses.staked_mint, account_addresses.vault, 100);
    let (user_staked_ata, user_staked_ata_account) =
        associated_token::create_account_for_associated_token_account(user_staked_token_state);

    let amount = 100u64;

    let mut data2 = anchor_discriminator("deposit").to_vec();
    data2.extend_from_slice(&amount.to_le_bytes());

    let ix2 = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(account_addresses.staked_mint, false),
            AccountMeta::new_readonly(account_addresses.admin, false),
            AccountMeta::new(user_staked_ata, false),
            AccountMeta::new(account_addresses.vault, false),
            AccountMeta::new(account_addresses.staked_mint_ata, false),
            AccountMeta::new(user_stake_profile, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(associated_token::ID, false),
        ],
        data: data2,
    };

    let (_, staked_mint_account) = mint_maker(account_addresses.admin);
    let admin_account = Account::new(1_000_000_000, 0, &solana_sdk_ids::system_program::id());
    let empty_system_account = Account::new(0, 0, &solana_sdk_ids::system_program::id());
    let input_accounts2 = vec![
        (user, user_account),
        (account_addresses.staked_mint, staked_mint_account),
        (account_addresses.admin, admin_account),
        (user_staked_ata, user_staked_ata_account),
        (account_addresses.vault, vault_after.clone()),
        (account_addresses.staked_mint_ata, staked_ata_after.clone()),
        (user_stake_profile, empty_system_account.clone()),
        keyed_account_for_system_program(),
        token::keyed_account(),
        associated_token::keyed_account(),
    ];

    let result2 = account_addresses
        .mollusk
        .process_instruction(&ix2, &input_accounts2);

    assert!(
        result2.program_result.is_ok(),
        "deposit failed: {:?}",
        result2.program_result
    );

    let vault_after_deposit = result2
        .get_account(&account_addresses.vault)
        .expect("vault is missing");

    let data2 = &vault_after_deposit.data;
    assert!(data2.len() >= 8 + 32 + 32 + 32 + 8 + 8 + 8 + 16 + 1);

    let body2 = &data2[8..];

    let total_staked_deposit = u64::from_le_bytes(body2[96..104].try_into().unwrap());
    let last_updated_deposit = i64::from_le_bytes(body2[104..112].try_into().unwrap());
    let reward_rate_stored_deposit = u64::from_le_bytes(body2[112..120].try_into().unwrap());
    let reward_per_token_stored_deposit = u128::from_le_bytes(body2[120..136].try_into().unwrap());

    assert!(total_staked_deposit == 100);
    assert!(reward_rate_stored_deposit == account_addresses.reward_rate);

    let user_stake_profile_deposit = result2
        .get_account(&user_stake_profile)
        .expect("user stake profile is missing");

    let user_data = &user_stake_profile_deposit.data;

    assert!(user_data.len() >= 8 + 32 + 8 + 8 + 16 + 8);

    let user_data_body = &user_data[8..];

    let mut user_authority_bytes = [0u8; 32];
    user_authority_bytes.copy_from_slice(&user_data_body[..32]);
    let user_staked = u64::from_le_bytes(user_data_body[32..40].try_into().unwrap());

    assert_eq!(Pubkey::new_from_array(user_authority_bytes), user);
    assert!(user_staked == 100);
}
