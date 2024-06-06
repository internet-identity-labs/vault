use async_trait::async_trait;
use candid::{CandidType};
use ic_cdk::id;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::impl_basic_for_transaction;
use crate::enums::{Currency, TransactionState, VaultRole};
use crate::enums::TransactionState::{Executed, Failed};
use crate::errors::VaultError;
use crate::errors::VaultError::CanisterReject;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::top_up_transaction::{calculate_cycle_minter_id, MEMO_TOP_UP_CANISTER, notify_top_up};
use crate::transaction::transfer::transfer_common::TransferCommon;
use crate::transfer_service::transfer;
/*
if you make any changes to this file
 you need to unskip and run
 top_up_transaction.test.ts
*/

impl_basic_for_transaction!(TopUpQuorumTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TopUpQuorumTransaction {
    common: BasicTransactionFields,
    wallet: String,
    block_index: Option<BlockIndex>,
    amount: u64,
    currency: Currency,
}

impl TopUpQuorumTransaction {
    fn new(state: TransactionState, currency: Currency,
           wallet: String, amount: u64, ) -> Self {
        TopUpQuorumTransaction {
            common: BasicTransactionFields::new(state, None, false),
            wallet,
            currency,
            block_index: None,
            amount,
        }
    }
}

#[async_trait]
impl ITransaction for TopUpQuorumTransaction {
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        self.get_transfer_block_predicate(tr)
    }

    fn get_accepted_roles(&self) -> Vec<VaultRole> {
        vec![VaultRole::Admin]
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        let state = get_current_state();
        let threshold = state.quorum.quorum;
        self.set_threshold(threshold.clone());
        Ok(threshold)
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let to = calculate_cycle_minter_id().await;
        let transfer = transfer(self.amount.clone(), to, self.wallet.clone(), Some(MEMO_TOP_UP_CANISTER))
            .await;
        match transfer {
            Ok(result) => {
                self.block_index = Some(result);
                let notify_res = notify_top_up(result.clone(), id()).await;
                match notify_res {
                    Ok(_) => {
                        self.set_state(Executed);
                    }
                    Err(message) => {
                        self.set_state(Failed);
                        self.get_common_mut().error = Some(CanisterReject { message });
                    }
                }
            }
            Err(message) => {
                self.set_state(Failed);
                self.get_common_mut().error = Some(CanisterReject { message });
            }
        }
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TopUpQuorumTransaction = self.clone();
        TransactionCandid::TopUpQuorumTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TopUpQuorumTransactionRequest {
    wallet: String,
    amount: u64,
    currency: Currency,
}

pub struct TopUpQuorumTransactionBuilder {
    request: TopUpQuorumTransactionRequest,
}

impl TopUpQuorumTransactionBuilder {
    pub fn init(request: TopUpQuorumTransactionRequest) -> Self {
        return TopUpQuorumTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for TopUpQuorumTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TopUpQuorumTransaction::new(
            state,
            self.request.currency.clone(),
            self.request.wallet.clone(),
            self.request.amount.clone(),
        );
        Box::new(trs)
    }
}


impl TransferCommon for TopUpQuorumTransaction {
    fn get_wallet(&self) -> String {
        self.wallet.clone()
    }

    fn get_amount(&self) -> u64 {
        self.amount.clone()
    }

    fn set_policy(&mut self, _: Option<String>) {
        self.set_state(Failed);
        self.common.error = Some(VaultError::CouldNotDefinePolicy);
    }
}


