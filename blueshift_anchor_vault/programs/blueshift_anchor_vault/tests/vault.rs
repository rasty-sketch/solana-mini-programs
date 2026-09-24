use std::{ assert_eq, println };

use anchor_lang::{
    prelude::*,
    solana_program::instruction::Instruction,
    system_program,
    InstructionData,
    ToAccountMetas,
};
use blueshift_anchor_vault::{ accounts, instruction, ID };
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;

#[test]
fn deposit_sol_in_vault() {
    let mut svm = LiteSVM::new();

    let program_path = std::path::Path
        ::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/blueshift_anchor_vault.so");

    svm.add_program_from_file(ID, program_path).expect("load program, build first");

    let payer = Keypair::new();
    let user = Keypair::new();

    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let user_key = user.pubkey();

    let (vault, _bump) = Pubkey::find_program_address(&[b"vault", user_key.as_ref()], &ID);

    let amount = svm.minimum_balance_for_rent_exemption(0) + 1;
    let user_before = svm.get_balance(&user_key).unwrap();

    assert_eq!(svm.get_balance(&vault.key()).unwrap_or(0), 0);

    let ix = Instruction {
        program_id: ID,
        accounts: (accounts::VaultAction {
            user: user_key,
            vault,
            system_program: system_program::ID,
        }).to_account_metas(None),
        data: (instruction::Deposit { amount }).data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &user],
        svm.latest_blockhash()
    );

    let res = svm.send_transaction(tx).expect("shouuld be able to deposit");
    println!("{:?}", res.logs);

    let vault_account = svm.get_account(&vault).unwrap();

    assert_eq!(vault_account.lamports, amount);
    assert_eq!(vault_account.owner, system_program::ID);
    assert!(vault_account.data.is_empty());

    assert_eq!(svm.get_balance(&user_key).unwrap(), user_before - amount);
}
