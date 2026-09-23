use anchor_lang::AccountDeserialize;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use cust_fb::FacebookAccount;
use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use solana_transaction::Transaction;

#[test]
fn test_initialize() {
    let (mut svm, user, facebook_pda) = set_up();

    let ix_data = cust_fb::instruction::Initialize {
        name: "rusty".to_string(),
        status: "building".to_string(),
        twitter: "rustydevv".to_string(),
    };

    let accounts = cust_fb::accounts::Initialize {
        user: user.pubkey(),
        facebook_account: facebook_pda,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: cust_fb::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };

    let blockhash = svm.latest_blockhash();

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], blockhash);

    let _result = svm.send_transaction(tx).unwrap();

    let account = svm.get_account(&facebook_pda).unwrap();

    let fb_data: FacebookAccount =
        FacebookAccount::try_deserialize(&mut account.data.as_slice()).unwrap();

    assert_eq!(fb_data.name, "rusty");
    assert_eq!(fb_data.status, "building");
    assert_eq!(fb_data.twitter, "rustydevv");
    assert_eq!(fb_data.authority, user.pubkey());
}

#[test]
fn update() {
    let (mut svm, user, pda) = set_up();

    let ix_data = cust_fb::instruction::Initialize {
        name: "rusty".to_string(),
        status: "building".to_string(),
        twitter: "rustydevv".to_string(),
    };

    let accounts = cust_fb::accounts::Initialize {
        user: user.pubkey(),
        facebook_account: pda,
        system_program: system_program::ID,
    };

    let ix = Instruction {
        program_id: cust_fb::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };

    let blockhash = svm.latest_blockhash();

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], blockhash);
    svm.send_transaction(tx).unwrap();

    let ix_data2 = cust_fb::instruction::Update {
        status: String::from("new status"),
    };

    let accounts2 = cust_fb::accounts::Update {
        user: user.pubkey(),
        facebook_account: pda,
    };

    let ix2 = Instruction {
        program_id: cust_fb::ID,
        accounts: accounts2.to_account_metas(None),
        data: ix_data2.data(),
    };

    let blockhash2 = svm.latest_blockhash();
    let tx2 =
        Transaction::new_signed_with_payer(&[ix2], Some(&user.pubkey()), &[&user], blockhash2);

    let _result = svm.send_transaction(tx2).unwrap();

    let account2 = svm.get_account(&pda).unwrap();

    let fb_data: FacebookAccount =
        FacebookAccount::try_deserialize(&mut account2.data.as_slice()).unwrap();

    assert_eq!("new status", fb_data.status);
}

pub fn set_up() -> (LiteSVM, Keypair, Pubkey) {
    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!("../../../target/sbpf-solana-solana/release/cust_fb.so");

    svm.add_program(cust_fb::ID, program_bytes).unwrap();

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    let (pda, _) = Pubkey::find_program_address(
        &[b"self-custodial-facebook2", user.pubkey().as_ref()],
        &cust_fb::ID,
    );

    (svm, user, pda)
}
