use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, Memo, Subaccount, Tokens};

use crate::config::CONF;
use crate::to_array;

pub async fn transfer(amount: u64, to: AccountIdentifier, from_hex: String, memo: Option<u64>) -> Result<BlockIndex, String> {
    let tokens = Tokens::from_e8s(amount);
    let from_decoded = hex::decode(from_hex).unwrap();
    let from_sub = Subaccount(to_array(from_decoded));
    let ledger_canister_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
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