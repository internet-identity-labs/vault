use candid::CandidType;
use candid::Principal;
use ic_cdk::api::call::{CallResult, RejectionCode};
use ic_cdk::call;
use ic_ledger_types::{AccountIdentifier, BlockIndex as BlockIndexLegacy, DEFAULT_FEE, MAINNET_LEDGER_CANISTER_ID, Memo as MemoLegacy, Subaccount as SubLegacy, Tokens};
use icrc_ledger_types::icrc1;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::account::Subaccount;
use icrc_ledger_types::icrc1::transfer::{BlockIndex, NumTokens, TransferError};
use serde::{Deserialize, Serialize};

use crate::to_array;

pub async fn transfer(amount: u64, to: AccountIdentifier, from_hex: String, memo: Option<u64>) -> Result<BlockIndexLegacy, String> {
    let tokens = Tokens::from_e8s(amount);
    let from_decoded = match hex::decode(from_hex) {
        Ok(x) => { x }
        Err(err) => {
            match err {
                hex::FromHexError::InvalidHexCharacter { c, index } => {
                    return Err(format!("Invalid hex character {} at index {}", c, index));
                }
                hex::FromHexError::OddLength => {
                    return Err("OddLength".to_string());
                }
                hex::FromHexError::InvalidStringLength => {
                    return Err("InvalidStringLength".to_string());
                }
            }
        }
    };
    let from_sub = SubLegacy(to_array(from_decoded));
    let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
    let memo = match memo {
        None => { MemoLegacy(0) }
        Some(u) => { MemoLegacy(u) }
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TransferResult { Ok(BlockIndex), Err(TransferError) }

pub async fn transfer_icrc1(icrc1_canister: Principal, amount: u64, to_owner: Principal, subaccount: Option<Subaccount>, from_wallet_hex: String) -> CallResult<(TransferResult, )> {
    let amount_nat = NumTokens::from(amount);
    let from_decoded = match hex::decode(from_wallet_hex) {
        Ok(x) => { x }
        Err(err) => {
            return Err((RejectionCode::DestinationInvalid, format!("Failed to decode hex: {:?}", err)));
        }
    };
    let from_sub = to_array(from_decoded);
    let args = icrc1::transfer::TransferArg {
        from_subaccount: Some(from_sub),
        to: Account { owner: to_owner, subaccount },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: amount_nat,
    };

    let result: CallResult<(TransferResult, )> = call(icrc1_canister, "icrc1_transfer", (args, )).await;
    result
}
