use async_trait::async_trait;
use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::enums::{Currency, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{get_vault_state_block_predicate, ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transfer_service::transfer;

impl_basic_for_transaction!(TransferTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferTransaction {
    common: BasicTransactionFields,
    policy: Option<String>,
    wallet: String,
    block_index: Option<BlockIndex>,
    amount: u64,
    currency: Currency,
    address: String,
}

impl TransferTransaction {
    fn new(state: TransactionState, address: String, currency: Currency,
           wallet: String, amount: u64, ) -> Self {
        TransferTransaction {
            common: BasicTransactionFields::new(state, None, TrType::Transfer, false),
            wallet,
            policy: None,
            currency,
            block_index: None,
            amount,
            address,
        }
    }
}

#[async_trait]
impl ITransaction for TransferTransaction {
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        if tr.get_id() >= self.get_id() {
            return false;
        }
        if get_vault_state_block_predicate(tr) {
            return true;
        }
        if let TransactionCandid::TransferTransactionV(transfer) = tr.to_candid() {
            return transfer.wallet == self.wallet;
        }
        false
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let transfer = transfer(self.amount, self.address.clone(), self.wallet.clone())
            .await;
        match transfer {
            Ok(result) => {
                self.block_index = Some(result);
                self.set_state(Executed);
            }
            Err(s) => {
                self.set_state(Rejected);
                self.get_common_mut().memo = Some(s);
            }
        }
        state
    }

    fn define_threshold(&mut self) -> Result<u8, String> {
        let state = get_current_state();
        let policy = state.policies.iter()
            .filter(|p| p.wallets.contains(&self.wallet))
            .filter(|p| p.amount_threshold < self.amount)
            .max_by(|a, b| {
                a.amount_threshold.cmp(&b.amount_threshold)
            });
        match policy {
            None => {
                Err("No suitable policy".to_string())
            }
            Some(x) => {
                self.policy = Some(x.uid.to_owned());
                self.set_threshold(x.member_threshold);
                Ok(x.member_threshold)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TransferTransaction = self.clone();
        TransactionCandid::TransferTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferTransactionRequest {
    wallet: String,
    amount: u64,
    currency: Currency,
    address: String,
}

pub struct TransferTransactionBuilder {
    request: TransferTransactionRequest,
}

impl TransferTransactionBuilder {
    pub fn init(request: TransferTransactionRequest) -> Self {
        return TransferTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for TransferTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TransferTransaction::new(
            state,
            self.request.address.clone(),
            self.request.currency.clone(),
            self.request.wallet.clone(),
            self.request.amount.clone(),
        );
        Box::new(trs)
    }
}


