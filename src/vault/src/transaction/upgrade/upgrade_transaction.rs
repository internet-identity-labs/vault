use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::call;
use ic_cdk::api::management_canister::main::{CanisterInstallMode, InstallCodeArgument};
use ic_cdk::id;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{impl_basic_for_transaction, VERSION};
use crate::config::get_repo_canister_id;
use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError::CanisterReject;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(VersionUpgradeTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct VersionUpgradeTransaction {
    common: BasicTransactionFields,
    version: String,
    initial_version: String,
}

impl VersionUpgradeTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, version: String) -> Self {
        VersionUpgradeTransaction {
            common: BasicTransactionFields::new(state, batch_uid, true),
            initial_version: VERSION.to_string(),
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
    async fn build_dyn_transaction(&mut self, mut state: TransactionState) -> Box<dyn ITransaction> {
        let initial_version = Version::parse(VERSION).unwrap();
        let expected_version = Version::parse(&self.request.version).unwrap();
        if expected_version <= initial_version {
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
        let initial_version = Version::parse(VERSION).unwrap();
        let expected_version = Version::parse(&self.version).unwrap();
        if expected_version <= initial_version {
            self.set_state(Executed);
            state
        } else {
            match upgrade_self(expected_version.to_string()).await {
                Ok(_) => {
                    self.set_state(Executed);
                }
                Err(msg) => {
                    self.set_state(Rejected);
                    self.get_common_mut().error = Some(CanisterReject { message: msg.clone() });
                }
            }
            state
        }
    }


    fn to_candid(&self) -> TransactionCandid {
        TransactionCandid::UpgradeTransactionV(self.clone())
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultWasm {
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
    version: String,
    hash: String,
}


async fn upgrade_self(version: String) -> Result<(), String> {
    let sem_ver = match Version::parse(&version) {
        Ok(x) => {
            x
        }
        Err(msg) => {
            return Err(format!(
                "Failed to parse semver!: {}",
                msg
            ));
        }
    };
    let (wasm, ): (VaultWasm, ) = match call::call(
        get_repo_canister_id(),
        "get_by_version",
        (sem_ver.to_string(), ),
    ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            ic_cdk::eprintln!("Error while getting wasm: [{:?}] {}", code, msg);
            return Err(format!(
                "An error happened during the get_by_version call: {}: {}",
                code as u8, msg
            ));
        }
    };
    let arg = InstallCodeArgument {
        mode: CanisterInstallMode::Upgrade,
        canister_id: id(),
        wasm_module: wasm.wasm_module,
        arg: vec![],
    };
    let result = ic_cdk::api::management_canister::main::install_code(arg).await;
    ic_cdk::println!("Upgrade result: {:?}", result);
    if let Err((code, msg)) = result {
        ic_cdk::eprintln!("Error while upgrading canister: [{:?}] {}", code, msg);
        return Err("Error while upgrading canister".to_string());
    }
    Ok(())
}