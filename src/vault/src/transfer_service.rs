use ic_cdk::{id, print};
use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens};

use crate::to_array;

pub async fn transfer(amount: u64, to: AccountIdentifier, from_hex: String, memo: Option<u64>) -> Result<BlockIndex, String> {
    let tokens = Tokens::from_e8s(amount);
    let from_decoded = hex::decode(from_hex).unwrap();
    let from_sub = Subaccount(to_array(from_decoded));
    let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
    let memo = match memo {
        None => {Memo(0)}
        Some(u) => {Memo(u)}
    };
    let transfer_args = ic_ledger_types::TransferArgs {
        memo,
        amount: tokens,
        fee: DEFAULT_FEE,
        from_subaccount: Some(from_sub),
        to,
        created_at_time: None,
    };
    ic_ledger_types::transfer(ledger_canister_id, transfer_args).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error: {:?}", e))
}

pub async fn transfer_opt(amount: u64, to: AccountIdentifier, memo: Option<u64>) -> Result<BlockIndex, String> {
    let tokens = Tokens::from_e8s(amount);
    let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
    let memo = match memo {
        None => {Memo(0)}
        Some(u) => {Memo(u)}
    };
    let account_id = ic_ledger_types::AccountIdentifier::new(&id(), &DEFAULT_SUBACCOUNT);
    print(format!("account_id: {:?}", account_id));
    let transfer_args = ic_ledger_types::TransferArgs {
        memo,
        amount: tokens,
        fee: DEFAULT_FEE,
        from_subaccount: None,
        to,
        created_at_time: None,
    };
    ic_ledger_types::transfer(ledger_canister_id, transfer_args).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error: {:?}", e))
}