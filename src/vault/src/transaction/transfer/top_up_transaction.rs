use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::id;
use ic_ledger_types::{AccountIdentifier, BlockIndex, Subaccount};
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, impl_transfer_common_for_transaction};
use crate::enums::{Currency, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError;
use crate::errors::VaultError::CanisterReject;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::transfer_common::TransferCommon;
use crate::transfer_service::transfer;

const CYCLE_MINTER_CANISTER_ID: &str = "rkp4c-7iaaa-aaaaa-aaaca-cai";
const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;

/*
if you make any changes to this file
 you need to unskip and run
 top_up_transaction.test.ts
*/

impl_basic_for_transaction!(TopUpTransaction);
impl_transfer_common_for_transaction!(TopUpTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TopUpTransaction {
    common: BasicTransactionFields,
    policy: Option<String>,
    wallet: String,
    block_index: Option<BlockIndex>,
    amount: u64,
    currency: Currency,
}

impl TopUpTransaction {
    fn new(state: TransactionState, currency: Currency,
           wallet: String, amount: u64, ) -> Self {
        TopUpTransaction {
            common: BasicTransactionFields::new(state, None, false),
            wallet,
            policy: None,
            currency,
            block_index: None,
            amount,
        }
    }
}

#[async_trait]
impl ITransaction for TopUpTransaction {
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        self.get_transfer_block_predicate(tr)
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        self.define_transfer_threshold()
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let cycle_minter_id = Principal::from_text(CYCLE_MINTER_CANISTER_ID).unwrap();
        let id = id();
        let to_subaccount = Subaccount::from(id);
        let to = AccountIdentifier::new(&cycle_minter_id, &to_subaccount);
        let transfer = transfer(self.amount, to, self.wallet.clone(), Some(MEMO_TOP_UP_CANISTER))
            .await;
        match transfer {
            Ok(result) => {
                self.block_index = Some(result);
                let notify_res = notify_top_up(result, id).await;
                match notify_res {
                    Ok(_) => {
                        self.set_state(Executed);
                    }
                    Err(message) => {
                        self.set_state(Rejected);
                        self.get_common_mut().error = Some(CanisterReject { message });
                    }
                }
            }
            Err(message) => {
                self.set_state(Rejected);
                self.get_common_mut().error = Some(CanisterReject { message });
            }
        }
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TopUpTransaction = self.clone();
        TransactionCandid::TopUpTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TopUpTransactionRequest {
    wallet: String,
    amount: u64,
    currency: Currency,
}

pub struct TopUpTransactionBuilder {
    request: TopUpTransactionRequest,
}

impl TopUpTransactionBuilder {
    pub fn init(request: TopUpTransactionRequest) -> Self {
        return TopUpTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for TopUpTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TopUpTransaction::new(
            state,
            self.request.currency.clone(),
            self.request.wallet.clone(),
            self.request.amount.clone(),
        );
        Box::new(trs)
    }
}


#[derive(CandidType)]
pub struct NotifyCanisterArgs {
    pub block_index: u64,
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum NotifyError {
    Refunded { block_index: Option<u64>, reason: String },
    InvalidTransaction(String),
    Other { error_message: String, error_code: u64 },
    Processing,
    TransactionTooOld(u64),
}


async fn notify_top_up(
    block_index: BlockIndex, id: Principal,
) -> Result<(), String> {
    let cycle_minter_id = Principal::from_text(CYCLE_MINTER_CANISTER_ID).unwrap();
    let args = NotifyCanisterArgs {
        block_index,
        canister_id: id,
    };
    let (notify_res, ) = ic_cdk::call::<_, (Result<u128, NotifyError>, )>(cycle_minter_id, "notify_top_up", (args, ))
        .await
        .map_err(|(code, msg)|
            format!("Notify top-up error: {}: {}. Params {} + {} ", code as u8, msg, block_index, id)
        ).unwrap();

    if let Err(e) = notify_res {
        return Err(format!("Error from {:?}", e));
    }
    Ok(())
}
