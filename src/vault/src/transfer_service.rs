use std::convert::TryFrom;
use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, Memo, Subaccount, Tokens};

use crate::memory::CONF;
use crate::to_array;

pub async fn transfer(amount: u64, address: String, from_hex: String) -> Result<BlockIndex, String> {
    let to_decoded = hex::decode(address).unwrap();
    let to: AccountIdentifier = AccountIdentifier::try_from(to_array(to_decoded)).unwrap();
    let tokens = Tokens::from_e8s(amount);
    let from_decoded = hex::decode(from_hex).unwrap();
    let from_sub = Subaccount(to_array(from_decoded));
    let ledger_canister_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
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