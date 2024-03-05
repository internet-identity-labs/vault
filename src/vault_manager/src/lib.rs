use std::cell::RefCell;

use api::call;
use candid::{export_service, Principal};
use candid::CandidType;
use ic_cdk::{api, call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::{CanisterInstallMode, CanisterSettings, InstallCodeArgument};
use ic_cdk_macros::*;
use ic_ledger_types::{GetBlocksArgs, MAINNET_LEDGER_CANISTER_ID, Operation, query_blocks};
pub use semver::Version;
use serde::{Deserialize, Serialize};

use nfid_certified::{CertifiedResponse, get_trusted_origins_cert, update_trusted_origins};

use crate::config::{Conf, CONF};

mod config;

const FEE: u128 = 100_000_000_000;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultCanister {
    canister_id: Principal,
    initiator: Principal,
    block_number: u64,
    vault_type: VaultType,
}

thread_local! {
    pub static CANISTERS: RefCell<Vec<VaultCanister>> = RefCell::new(Vec::default());
}

#[derive(CandidType, Clone, Deserialize)]
struct CreateCanisterArgs<TCycles> {
    cycles: TCycles,
    settings: CanisterSettings,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultWasm {
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
    version: String,
    hash: String,
}

#[derive(CandidType, Deserialize)]
struct CreateResult {
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct WalletStoreWASMArgs {
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
}

// Install Wasm
#[derive(CandidType, Deserialize)]
enum InstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
enum VaultType {
    Pro,
    Light,
}

#[init]
async fn init(conf: Conf) {
    update_trusted_origins(conf.origins.clone());
    CONF.with(|c| c.replace(conf));
}

#[query]
async fn get_all_canisters() -> Vec<VaultCanister> {
    CANISTERS.with(|c| c.borrow().clone())
}

#[update]
async fn create_canister_call(block_number: u64, vault_type: Option<VaultType>, owner: Option<Principal>) -> Result<CreateResult, String> {
    let vault_type = vault_type.unwrap_or_else(|| VaultType::Pro);

    verify_payment(block_number).await;

    let set = CanisterSettings {
        controllers: Some(vec![id()]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };

    let args = CreateCanisterArgs {
        cycles: get_initial_cycles_balance() + FEE,
        settings: set.clone(),
    };

    #[derive(CandidType)]
    struct In {
        settings: Option<CanisterSettings>,
    }

    let in_arg = In {
        settings: Some(set),
    };

    let (create_result, ): (CreateResult, ) = match call::call_with_payment128(
        Principal::management_canister(),
        "create_canister",
        (in_arg, ),
        args.cycles,
    ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "Failed to create canister!: {}: {}",
                code as u8, msg
            ));
        }
    };

    let initiator = owner.unwrap_or_else(|| caller());
    install_wallet(&create_result.canister_id, &initiator).await?;

    CANISTERS.with(|c| c.borrow_mut().push(VaultCanister {
        canister_id: create_result.canister_id.clone(),
        initiator,
        block_number: block_number.clone(),
        vault_type,
    }));

    let _ = update_settings(UpdateSettingsArg {
        canister_id: create_result.canister_id.clone(),
        settings: CanisterSettings {
            controllers: Some(vec![create_result.canister_id]),
            compute_allocation: None,
            freezing_threshold: None,
            memory_allocation: None,
        },
    }).await;
    Ok(create_result)
}

#[derive(Clone, CandidType, Deserialize)]
pub struct UpdateSettingsArg {
    pub canister_id: Principal,
    pub settings: CanisterSettings,
}

pub async fn update_settings(args: UpdateSettingsArg) -> CallResult<((), )> {
    let update_settings_result =
        call(Principal::management_canister(), "update_settings", (args, )).await;
    return update_settings_result;
}

async fn install_wallet(canister_id: &Principal, initiator: &Principal) -> Result<(), String> {
    #[derive(CandidType, Deserialize, Clone, Debug)]
    pub struct Conf {
        pub origins: Vec<String>,
        pub repo_canister: String,
    }

    let conf_manager = CONF.with(|c| c.borrow().clone());
    let conf = Conf {
        origins: conf_manager.origins,
        repo_canister: conf_manager.repo_canister_id,
    };

    let arg = match candid::encode_args((initiator, conf)) {
        Ok(a) => { a }
        Err(msg) => {
            return Err(format!(
                "An error happened during an arg encoding: {}: {}",
                initiator.to_text(), msg
            ));
        }
    };

    let (wasm, ): (VaultWasm, ) = match call::call(
        get_repo_canister_id(),
        "get_latest_version",
        (),
    ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the get_by_version call: {}: {}",
                code as u8, msg
            ));
        }
    };

    let arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm.wasm_module,
        arg,
    };

    match call::call(
        Principal::management_canister(),
        "install_code",
        (arg, ), ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the install_code call: {}: {}",
                code as u8, msg
            ));
        }
    };

    Ok(())
}

#[query]
async fn canister_balance() -> u64 {
    ic_cdk::api::canister_balance()
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    canisters: Vec<VaultCanisterMemory>,
    config: Option<Conf>,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultCanisterMemory {
    canister_id: Principal,
    initiator: Principal,
    block_number: u64,
    vault_type: Option<VaultType>,
}

#[pre_upgrade]
pub fn stable_save() {
    let vaults: Vec<VaultCanister> = CANISTERS.with(|vaultss| {
        vaultss.borrow().clone()
    });
    let conf: Conf = CONF.with(|c| {
        c.borrow().clone()
    });

    let vaults_memory = vaults.iter().map(|x| VaultCanisterMemory {
        canister_id: x.canister_id.clone(),
        initiator: x.initiator.clone(),
        block_number: x.block_number.clone(),
        vault_type: Some(x.vault_type.clone()),
    }).collect();

    let mem = Memory {
        canisters: vaults_memory,
        config: Some(conf),
    };
    storage::stable_save((mem, )).unwrap();
}

#[post_upgrade]
pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    let vaults: Vec<VaultCanister> = mo.canisters.iter().map(|x| VaultCanister {
        canister_id: x.canister_id.clone(),
        initiator: x.initiator.clone(),
        block_number: x.block_number.clone(),
        vault_type: x.vault_type.clone().unwrap_or_else(|| VaultType::Pro),
    }).collect();

    CANISTERS.with(|c| c.borrow_mut().extend(
        vaults)
    );
    match mo.config {
        None => {}
        Some(conf) => {
            CONF.with(|vv| {
                update_trusted_origins(conf.origins.clone());
                vv.replace(conf);
            });
        }
    }
}


#[test]
fn sub_account_test() {}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}

#[update]
async fn get_trusted_origins() -> Vec<String> {
    CONF.with(|c| c.borrow().clone().origins)
}

#[query]
async fn get_config() -> Conf {
    CONF.with(|c| c.borrow().clone())
}

async fn verify_payment(block_number: u64) {
    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let canister = canisters.iter().find(|x| x.block_number == block_number);
        match canister {
            None => {}
            Some(_) => {
                trap("Block already used");
            }
        }
    });

    let args: GetBlocksArgs = GetBlocksArgs {
        start: block_number,
        length: 1,
    };

    let response = query_blocks(MAINNET_LEDGER_CANISTER_ID, args).await;

    match response {
        Ok(x) => {
            if x.blocks.len() == 0 {
                trap("No blocks found");
            }
            let operation = x.blocks[0].transaction.operation.clone().unwrap();
            match operation {
                Operation::Transfer { to, amount, .. } => {
                    if to.to_string() != get_destination_address() {
                        trap("Incorrect destination");
                    }
                    if amount.e8s() < get_payment_cycles() {
                        trap("Incorrect amount");
                    }
                }
                _ => {
                    trap("Operation is not Transfer");
                }
            }
        }
        Err(e) => {
            trap(format!("Error: {:?}", e).as_str());
        }
    }
}

fn get_repo_canister_id() -> Principal {
    CONF.with(|c| Principal::from_text(c.borrow().repo_canister_id.clone()).unwrap())
}

fn get_initial_cycles_balance() -> u128 {
    CONF.with(|c| c.borrow().initial_cycles_balance)
}

fn get_destination_address() -> String {
    CONF.with(|c| c.borrow().destination_address.clone())
}

fn get_payment_cycles() -> u64 {
    CONF.with(|c| c.borrow().icp_price)
}


#[query]
async fn get_trusted_origins_certified() -> CertifiedResponse {
    get_trusted_origins_cert()
}