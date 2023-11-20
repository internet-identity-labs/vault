use std::collections::HashSet;

use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::transaction::transaction::{ITransaction, TrType};
use crate::transaction::transactions_service::get_id;
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct BasicTransactionFields {
    pub id: u64,
    pub approves: HashSet<Approve>,
    pub state: TransactionState,
    pub initiator: String,
    pub created_date: u64,
    pub modified_date: u64,
    pub memo: Option<String>,
    pub transaction_type: TrType,
    pub is_vault_state: bool,
    pub batch_uid: Option<String>
}

impl BasicTransactionFields {
    pub fn new(state: TransactionState
               , tr_type: TrType, is_vault_state: bool,
    ) -> Self {
        BasicTransactionFields {
            id: get_id(),
            is_vault_state,
            approves: Default::default(),
            state,
            initiator: caller_to_address(),
            created_date: time(),
            modified_date: time(),
            memo: None,
            transaction_type: tr_type,
            batch_uid: None,
        }
    }
}


pub trait BasicTransaction  {
    fn get_common_mut(&mut self) -> &mut BasicTransactionFields;
    fn get_common_ref(&self) -> &BasicTransactionFields;
    fn clone_self(&self) -> Box<dyn ITransaction>;
    fn get_id(&self) -> u64 {
        self.get_common_ref().id
    }
    fn get_state(&self) -> &TransactionState {
        &self.get_common_ref().state
    }
    fn is_vault_state(&self) -> bool {
        self.get_common_ref().is_vault_state
    }
    fn store_approve(&mut self, approve: Approve) {
        if !self.get_common_ref().approves.contains(&approve) {
            self.get_common_mut().approves.insert(approve);
        }
    }
    fn set_state(&mut self, ts: TransactionState) {
        self.get_common_mut().state = ts
    }
    fn get_type(&self) -> &TrType {
        &self.get_common_ref().transaction_type
    }
}

#[macro_export]
macro_rules! impl_basic_for_transaction {
    ($type:ty) => {
        impl BasicTransaction for $type {
            fn get_common_mut(&mut self) -> &mut BasicTransactionFields {
                &mut self.common
            }

            fn get_common_ref(&self) -> &BasicTransactionFields {
                &self.common
            }

            fn clone_self(&self) -> Box<dyn ITransaction> {
                Box::new(self.clone())
            }
        }
    };
}
