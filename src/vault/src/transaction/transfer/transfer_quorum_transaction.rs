use async_trait::async_trait;
use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, impl_transfer_executor_for_transaction};
use crate::enums::{Currency, TransactionState, VaultRole};
use crate::enums::TransactionState::Rejected;
use crate::errors::VaultError;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::transfer_executor_common::TransferExecutor;
use crate::transaction::transfer::transfer_common::TransferCommon;


impl_basic_for_transaction!(TransferQuorumTransaction);
impl_transfer_executor_for_transaction!(TransferQuorumTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferQuorumTransaction {
    common: BasicTransactionFields,
    wallet: String,
    block_index: Option<BlockIndex>,
    amount: u64,
    currency: Currency,
    address: String,
}


impl TransferQuorumTransaction {
    fn new(state: TransactionState, address: String, currency: Currency,
           wallet: String, amount: u64, memo: Option<String>) -> Self {
        let mut common = BasicTransactionFields::new(state, None, false);
        common.memo = memo;
        TransferQuorumTransaction {
            common,
            wallet,
            currency,
            block_index: None,
            amount,
            address,
        }
    }
}

#[async_trait]
impl ITransaction for TransferQuorumTransaction {
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        self.get_transfer_block_predicate(tr)
    }

    fn get_accepted_roles(&self) -> Vec<VaultRole> {
        vec![VaultRole::Admin]
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        let state = get_current_state();
        Ok(state.quorum.quorum)
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TransferQuorumTransaction = self.clone();
        TransactionCandid::TransferQuorumTransactionV(trs)
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        self.execute_transfer(state).await
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferQuorumTransactionRequest {
    wallet: String,
    amount: u64,
    currency: Currency,
    address: String,
    memo: Option<String>,
}

pub struct TransferQuorumTransactionBuilder {
    request: TransferQuorumTransactionRequest,
}

impl TransferQuorumTransactionBuilder {
    pub fn init(request: TransferQuorumTransactionRequest) -> Self {
        return TransferQuorumTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for TransferQuorumTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TransferQuorumTransaction::new(
            state,
            self.request.address.clone(),
            self.request.currency.clone(),
            self.request.wallet.clone(),
            self.request.amount.clone(),
            self.request.memo.clone(),
        );
        Box::new(trs)
    }
}


impl TransferCommon for TransferQuorumTransaction {
    fn get_wallet(&self) -> String {
        self.wallet.clone()
    }

    fn get_amount(&self) -> u64 {
        self.amount.clone()
    }

    fn set_policy(&mut self, x: Option<String>) {
        self.set_state(Rejected);
        self.common.error = Some(VaultError::CouldNotDefinePolicy);
    }
}



