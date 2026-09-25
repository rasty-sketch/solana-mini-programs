// use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
// use mollusk_svm_programs_token::{associated_token, token};
// use sha2::{Digest, Sha256};
// use solana_account::Account;
// use solana_instruction::{AccountMeta, Instruction};
// use solana_program_option::COption;
// use solana_program_pack::Pack;
// use solana_pubkey::Pubkey;
// use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
// use spl_token_interface::state::Mint;

// const PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("52hdHGSANrycFt1ku1qy4GWBE9cdKTRyaexTsu17SQ36");

// fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
//     // Anchor serializes each instruction with an 8-byte "discriminator" first.
//     // This discriminator tells the on-chain program which method to route to.
//     // Formula used by Anchor:
//     //   discriminator = first_8_bytes(sha256("global:<instruction_name>"))
//     // For this test we manually build instruction bytes, so we compute it here.
//     let mut hasher = Sha256::new();
//     hasher.update(format!("global:{ix_name}"));
//     let hash = hasher.finalize();
//     let mut disc = [0u8; 8];
//     disc.copy_from_slice(&hash[..8]);
//     disc
// }

// #[test]
// fn invariant_vault_init_sets_expected_state() {
//     // Mollusk loads program ELF files from search paths.
//     // We set SBF_OUT_DIR so `Mollusk::new(&PROGRAM_ID, "token_staking_rewards_vault")`
//     // can find `<SBF_OUT_DIR>/token_staking_rewards_vault.so` reliably.
//     let sbf_out_dir = format!("{}/../../target/deploy", env!("CARGO_MANIFEST_DIR"));
//     std::env::set_var("SBF_OUT_DIR", sbf_out_dir);

//     // Create the local SVM harness with your staking program loaded.
//     // This does NOT start a validator; it runs one instruction in-process.
//     let mut mollusk = Mollusk::new(&PROGRAM_ID, "token_staking_rewards_vault");

//     // `vault_init` performs CPI calls to:
//     // - SPL Token Program
//     // - SPL Associated Token Program
//     // In Mollusk, CPI targets must also be present in the program cache.
//     token::add_program(&mut mollusk);
//     associated_token::add_program(&mut mollusk);

//     // Generate test pubkeys for actors/mints.
//     // These are just addresses; account state is provided separately below.
//     let admin = Pubkey::new_unique();
//     let staked_mint = Pubkey::new_unique();
//     let reward_mint = Pubkey::new_unique();

//     // Re-derive the same PDA as your program expects:
//     // seeds = [b"vault", admin.as_ref()], program_id = PROGRAM_ID
//     // We pass this exact address in the accounts list for `vault_init`.
//     let (vault, _vault_bump) =
//         Pubkey::find_program_address(&[b"vault", admin.as_ref()], &PROGRAM_ID);

//     // Derive the two ATAs that `vault_init` is expected to initialize:
//     // - vault's staked-mint ATA
//     // - vault's reward-mint ATA
//     // These addresses are deterministic from (authority, mint, token_program_id).
//     let staked_mint_ata =
//         get_associated_token_address_with_program_id(&vault, &staked_mint, &token::ID);
//     let reward_mint_ata =
//         get_associated_token_address_with_program_id(&vault, &reward_mint, &token::ID);

//     // Build raw instruction data expected by Anchor:
//     // [8-byte discriminator][reward_rate: u64 little-endian]
//     let reward_rate = 5_u64;
//     let mut data = anchor_discriminator("vault_init").to_vec();
//     data.extend_from_slice(&reward_rate.to_le_bytes());

//     // Build instruction metas in the EXACT order Anchor generated for
//     // `Context<Vaultinit>`. If order/flags differ, account validation fails.
//     //
//     // Writable/signer flags here mirror your account constraints:
//     // - admin is signer + mutable payer
//     // - vault and ATAs are mutable because they are initialized
//     // - program IDs/mints are readonly
//     let ix = Instruction {
//         program_id: PROGRAM_ID,
//         accounts: vec![
//             AccountMeta::new(admin, true),
//             AccountMeta::new_readonly(staked_mint, false),
//             AccountMeta::new_readonly(reward_mint, false),
//             AccountMeta::new(vault, false),
//             AccountMeta::new(staked_mint_ata, false),
//             AccountMeta::new(reward_mint_ata, false),
//             AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
//             AccountMeta::new_readonly(token::ID, false),
//             AccountMeta::new_readonly(associated_token::ID, false),
//         ],
//         data,
//     };

//     // Create runtime account data for `admin`.
//     // This account must have enough lamports because it pays rent for:
//     // - vault PDA account
//     // - two ATA accounts
//     let admin_account = Account::new(10_000_000_000, 0, &solana_sdk_ids::system_program::id());

//     // Prepare two initialized SPL Mint accounts.
//     // Your constraints only require valid mint accounts; this is sufficient.
//     let mint_state = Mint {
//         mint_authority: COption::Some(admin),
//         supply: 1_000_000_000,
//         decimals: 6,
//         is_initialized: true,
//         freeze_authority: COption::None,
//     };

//     let staked_mint_account = token::create_account_for_mint(mint_state);
//     let reward_mint_account = token::create_account_for_mint(mint_state);

//     // Placeholder zeroed system accounts for items created by `init`/`init_if_needed`.
//     // After execution, these should be transformed into initialized program accounts.
//     let empty_system_account = Account::new(0, 0, &solana_sdk_ids::system_program::id());

//     // The execution account map Mollusk uses.
//     // Think of this as the in-memory "account database" for this one instruction.
//     //
//     // We include:
//     // - data accounts touched by the instruction
//     // - system program (for create_account)
//     // - token + associated token programs (for CPI)
//     let input_accounts = vec![
//         (admin, admin_account),
//         (staked_mint, staked_mint_account),
//         (reward_mint, reward_mint_account),
//         (vault, empty_system_account.clone()),
//         (staked_mint_ata, empty_system_account.clone()),
//         (reward_mint_ata, empty_system_account),
//         keyed_account_for_system_program(),
//         token::keyed_account(),
//         associated_token::keyed_account(),
//     ];

//     // Execute one instruction and capture post-execution account snapshots.
//     let result = mollusk.process_instruction(&ix, &input_accounts);
//     assert!(
//         result.program_result.is_ok(),
//         "vault_init failed: {:?}",
//         result.program_result
//     );

//     // -----------------------------
//     // Invariant 1: Vault ownership
//     // -----------------------------
//     // The vault account should now exist and be owned by staking program.
//     // If owner is wrong, your program cannot deserialize/mutate it later.
//     let vault_after = result.get_account(&vault).expect("vault account missing");
//     assert_eq!(vault_after.owner, PROGRAM_ID);

//     // -------------------------------------------
//     // Invariant 2: Vault fields after initialization
//     // -------------------------------------------
//     // We decode your `Vault` account payload manually to prove initialized
//     // values are correct and stable.
//     let data = &vault_after.data;
//     assert!(data.len() >= 8 + 32 + 32 + 32 + 8 + 8 + 8 + 16 + 1);
//     // Anchor account data starts with an 8-byte account discriminator.
//     // The remaining bytes are the serialized struct fields.
//     let body = &data[8..];

//     let mut admin_bytes = [0_u8; 32];
//     admin_bytes.copy_from_slice(&body[0..32]);
//     let mut staked_mint_bytes = [0_u8; 32];
//     staked_mint_bytes.copy_from_slice(&body[32..64]);
//     let mut reward_mint_bytes = [0_u8; 32];
//     reward_mint_bytes.copy_from_slice(&body[64..96]);

//     let total_staked = u64::from_le_bytes(body[96..104].try_into().unwrap());
//     let last_updated = i64::from_le_bytes(body[104..112].try_into().unwrap());
//     let reward_rate_stored = u64::from_le_bytes(body[112..120].try_into().unwrap());
//     let reward_per_token_stored = u128::from_le_bytes(body[120..136].try_into().unwrap());

//     assert_eq!(Pubkey::new_from_array(admin_bytes), admin);
//     assert_eq!(Pubkey::new_from_array(staked_mint_bytes), staked_mint);
//     assert_eq!(Pubkey::new_from_array(reward_mint_bytes), reward_mint);
//     assert_eq!(total_staked, 0);
//     assert_eq!(last_updated, 0);
//     assert_eq!(reward_rate_stored, reward_rate);
//     assert_eq!(reward_per_token_stored, 0);

//     // ----------------------------------------
//     // Invariant 3: ATA creation and token owner
//     // ----------------------------------------
//     // Both derived ATA addresses must now contain token-account state.
//     // Owner must be SPL Token program, not system/staking program.
//     let staked_ata_after = result
//         .get_account(&staked_mint_ata)
//         .expect("staked ATA missing");
//     let reward_ata_after = result
//         .get_account(&reward_mint_ata)
//         .expect("reward ATA missing");

//     assert_eq!(staked_ata_after.owner, token::ID);
//     assert_eq!(reward_ata_after.owner, token::ID);

//     // Extra sanity: unpack succeeds as SPL token account layout.
//     // This catches malformed ATA initialization.
//     let _ = spl_token_interface::state::Account::unpack(&staked_ata_after.data).unwrap();
//     let _ = spl_token_interface::state::Account::unpack(&reward_ata_after.data).unwrap();
// }
