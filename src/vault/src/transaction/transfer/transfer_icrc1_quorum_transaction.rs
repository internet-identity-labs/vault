use async_trait::async_trait;
use candid::{CandidType, Principal};
use icrc1::transfer::BlockIndex;
use icrc_ledger_types::icrc1;
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::{Deserialize, Serialize};

use crate::enums::{TransactionState, VaultRole};
use crate::enums::TransactionState::{Executed, Failed};
use crate::errors::VaultError;
use crate::errors::VaultError::CanisterReject;
use crate::impl_basic_for_transaction;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::transfer_common::TransferCommon;
use crate::transfer_service::{transfer_icrc1, TransferResult};

impl_basic_for_transaction!(TransferICRC1QuorumTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferICRC1QuorumTransaction {
    common: BasicTransactionFields,
    wallet: String,
    block_index: Option<BlockIndex>,
    amount: u64,
    ledger_id: Principal,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
}


impl TransferICRC1QuorumTransaction {
    fn new(state: TransactionState, to_principal: Principal, to_subaccount: Option<Subaccount>,
           wallet: String, amount: u64, ledger_id: Principal, vault_memo: Option<String>) -> Self {
        let mut common = BasicTransactionFields::new(state, None, false);
        common.memo = vault_memo;
        TransferICRC1QuorumTransaction {
            common,
            wallet,
            to_principal,
            block_index: None,
            amount,
            to_subaccount,
            ledger_id,
        }
    }
}

#[async_trait]
impl ITransaction for TransferICRC1QuorumTransaction {
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        self.get_transfer_block_predicate(tr)
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        let state = get_current_state();
        let t = state.quorum.quorum;
        self.set_threshold(t.clone());
        Ok(t)
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let transfer = transfer_icrc1(self.ledger_id,
                                      self.amount.clone(),
                                      self.to_principal,
                                      self.to_subaccount.clone(),
                                      self.wallet.clone())
            .await;
        match transfer {
            Ok(result) => {
                match result.0 {
                    TransferResult::Ok(x) => {
                        self.block_index = Some(x);
                        self.set_state(Executed);
                    }
                    TransferResult::Err(message) => {
                        self.set_state(Failed);
                        self.get_common_mut().error = Some(CanisterReject {
                            message: message.to_string()
                        });
                    }
                }
            }
            Err(message) => {
                self.set_state(Failed);
                self.get_common_mut().error = Some(CanisterReject {
                    message: message.1,
                });
            }
        }
        state
    }

    fn get_accepted_roles(&self) -> Vec<VaultRole> {
        vec![VaultRole::Admin]
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TransferICRC1QuorumTransaction = self.clone();
        TransactionCandid::TransferICRC1QuorumTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferICRC1QuorumTransactionRequest {
    wallet: String,
    amount: u64,
    ledger_id: Principal,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
    memo: Option<String>,
}

pub struct TransferICRC1QuorumTransactionBuilder {
    request: TransferICRC1QuorumTransactionRequest,
}

impl TransferICRC1QuorumTransactionBuilder {
    pub fn init(request: TransferICRC1QuorumTransactionRequest) -> Self {
        return TransferICRC1QuorumTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for TransferICRC1QuorumTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TransferICRC1QuorumTransaction::new(
            state,
            self.request.to_principal.clone(),
            self.request.to_subaccount.clone(),
            self.request.wallet.clone(),
            self.request.amount.clone(),
            self.request.ledger_id.clone(),
            self.request.memo.clone(),
        );
        Box::new(trs)
    }
}


impl TransferCommon for TransferICRC1QuorumTransaction {
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





