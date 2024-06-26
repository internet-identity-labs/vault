use async_trait::async_trait;
use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, impl_transfer_executor_for_transaction, impl_transfer_common_for_transaction};
use crate::enums::{Currency, TransactionState};
use crate::errors::VaultError;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::transfer_executor_common::TransferExecutor;
use crate::transaction::transfer::transfer_common::TransferCommon;

impl_transfer_executor_for_transaction!(TransferTransaction);
impl_basic_for_transaction!(TransferTransaction);
impl_transfer_common_for_transaction!(TransferTransaction);
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
           wallet: String, amount: u64, memo: Option<String>) -> Self {
        let mut common = BasicTransactionFields::new(state, None, false);
        common.memo = memo;
        TransferTransaction {
            common,
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
        self.get_transfer_block_predicate(tr)
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        self.define_transfer_threshold()
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: TransferTransaction = self.clone();
        TransactionCandid::TransferTransactionV(trs)
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        self.execute_transfer(state).await
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferTransactionRequest {
    wallet: String,
    amount: u64,
    currency: Currency,
    address: String,
    memo: Option<String>,
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
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = TransferTransaction::new(
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



