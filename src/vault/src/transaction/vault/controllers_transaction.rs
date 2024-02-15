use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::main::{CanisterSettings, update_settings, UpdateSettingsArgument};
use ic_cdk::{id};
use ic_cdk::api::call::{RejectionCode};
use serde::{Deserialize, Serialize};

use crate::{get_controllers, impl_basic_for_transaction};
use crate::enums::{TransactionState, VaultRole};
use crate::enums::VaultRole::Admin;
use crate::errors::VaultError;
use crate::errors::VaultError::ControllersUpdateError;
use crate::state::{get_current_state, VaultState};
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(ControllersUpdateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ControllersUpdateTransaction {
    common: BasicTransactionFields,
    current_controllers: Vec<Principal>,
    principals: Vec<Principal>,
}

impl ControllersUpdateTransaction {
    fn new(state: TransactionState, principals: Vec<Principal>,  current_controllers: Vec<Principal>,) -> Self {
        ControllersUpdateTransaction {
            common: BasicTransactionFields::new(state, None, true),
            current_controllers,
            principals,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ControllersUpdateTransactionRequest {
    principals: Vec<Principal>,
}

pub struct ControllersUpdateTransactionBuilder {
    request: ControllersUpdateTransactionRequest,
}

impl ControllersUpdateTransactionBuilder {
    pub fn init(request: ControllersUpdateTransactionRequest) -> Self {
        return ControllersUpdateTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for ControllersUpdateTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let current_controllers = get_controllers().await;
        let trs = ControllersUpdateTransaction::new(
            state,
            self.request.principals.clone(),
            current_controllers
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for ControllersUpdateTransaction {
    fn get_accepted_roles(&self) -> Vec<VaultRole> {
        return vec![Admin];
    }

    fn get_block_predicate(&mut self, _: &Box<dyn ITransaction>) -> bool {
        false
    }

    fn define_threshold(&mut self) -> Result<u8, VaultError> {
        let st = get_current_state();
        let mut t = st.quorum.quorum;
        let admins = st.members.iter()
            .filter(|m| m.role.eq(&Admin))
            .count() as u8;
        if admins < t {
            t = admins;
        }
        Ok(t)
    }

    async fn execute(&mut self, state: VaultState) -> VaultState {
        let controllers = get_controllers().await;
        self.current_controllers = controllers.clone();

        let result: Result<(), (RejectionCode, String)>    = update_settings(UpdateSettingsArgument {
            canister_id: id(),
            settings: CanisterSettings {
                controllers: Some(self.principals.clone()),
                compute_allocation: None,
                freezing_threshold: None,
                memory_allocation: None,
            },
        }).await;
        match result {
            Ok(_) => {
                self.set_state(TransactionState::Executed);
                state
            }
            Err((_, msg)) => {
                self.set_state(TransactionState::Rejected);
                self.common.error = Some(ControllersUpdateError { message: msg, });
                state
            }
        }
    }


    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::ControllersUpdateTransactionV(self.clone())
    }
}