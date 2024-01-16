use std::convert::TryFrom;

use async_trait::async_trait;
use candid::CandidType;
use ic_ledger_types::{AccountIdentifier, BlockIndex};
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, impl_transfer_for_transaction};
use crate::enums::{Currency, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError;
use crate::errors::VaultError::CanisterReject;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::transfer::transfer_common::TransferHelper;
use crate::transfer_service::transfer;
use crate::util::to_array;

impl_basic_for_transaction!(TransferTransaction);
impl_transfer_for_transaction!(TransferTransaction);
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

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let to_decoded = hex::decode(self.address.clone()).unwrap();
        let to: AccountIdentifier = AccountIdentifier::try_from(to_array(to_decoded)).unwrap();
        let transfer = transfer(self.amount, to, self.wallet.clone(), None)
            .await;
        match transfer {
            Ok(result) => {
                self.block_index = Some(result);
                self.set_state(Executed);
            }
            Err(message) => {
                self.set_state(Rejected);
                self.get_common_mut().error = Some(CanisterReject { message });
            }
        }
        state
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
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
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



