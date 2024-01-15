use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::call;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, VERSION};
use crate::config::get_management_canister;
use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError::CanisterReject;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(VersionUpgradeTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct VersionUpgradeTransaction {
    common: BasicTransactionFields,
    version: String,
}

impl VersionUpgradeTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, version: String) -> Self {
        VersionUpgradeTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::VersionUpgrade, true),
            version,
        }
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VersionUpgradeTransactionRequest {
    version: String,
}

pub struct VersionUpgradeTransactionBuilder {
    request: VersionUpgradeTransactionRequest,
}

impl VersionUpgradeTransactionBuilder {
    pub fn init(request: VersionUpgradeTransactionRequest) -> Self {
        return VersionUpgradeTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for VersionUpgradeTransactionBuilder {
    fn build_dyn_transaction(&mut self, mut state: TransactionState) -> Box<dyn ITransaction> {
        let current_version = Version::parse(VERSION).unwrap();
        let expected_version = Version::parse(&self.request.version).unwrap();
        if expected_version <= current_version {
            state = Rejected;
        }
        let trs = VersionUpgradeTransaction::new(
            state, None, self.request.version.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for VersionUpgradeTransaction {
    async fn execute(&mut self, state: VaultState) -> VaultState {
        let current_version = Version::parse(VERSION).unwrap();
        let expected_version = Version::parse(&self.version).unwrap();
        if expected_version <= current_version {
            self.set_state(Executed);
            state
        } else {
            let (_, state): ((), VaultState) = match call::call(
                get_management_canister(),
                "update_canister_self",
                (self.version.to_string(), ),
            ).await {
                Ok(x) => {
                    self.set_state(Executed);
                    (x, state)
                }
                Err((_, msg)) => {
                    self.set_state(Rejected);
                    self.get_common_mut().error = Some(CanisterReject { message: msg.clone() });
                    ((), state)
                }
            };
            state
        }
    }


    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::UpgradeTransactionV(self.clone())
    }
}