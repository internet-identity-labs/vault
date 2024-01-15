mod config;

use std::cell::RefCell;

use api::call;
use candid::{export_service, Principal};
use candid::CandidType;
use ic_cdk::{api, caller, id, storage, trap};
use ic_cdk::api::management_canister::main::{CanisterInstallMode, CanisterSettings, InstallCodeArgument};
use ic_cdk_macros::*;
use ic_ledger_types::{GetBlocksArgs, MAINNET_LEDGER_CANISTER_ID, Operation, query_blocks};
pub use semver::Version;
use serde::{Deserialize, Serialize};
use crate::config::{Conf, CONF};

const FEE: u128 = 100_000_000_000;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultCanister {
    canister_id: Principal,
    initiator: Principal,
    block_number: u64,
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

// #[init]
// async fn init(conf: Conf) {
//     CONF.with(|c| c.replace(conf));
// }

#[update]
async fn get_all_canisters() -> Vec<VaultCanister> {
    CANISTERS.with(|c| c.borrow().clone())
}

#[update]
async fn update_canister_self(version: String) -> Result<(), String> {
    let canister = caller();
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
        canister_id: canister,
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

#[update]
async fn create_canister_call(block_number: u64) -> Result<CreateResult, String> {
    verify_payment(block_number).await;

    let set = CanisterSettings {
        //TODO add this canister as a controller + for now add a debug dude
        controllers: Some(vec![id(), Principal::from_text("lh6kg-7ebfk-bwa26-zgl6l-l27vx-xnnr4-ow2n4-mm4cq-tfjky-rs5gq-5ae".to_string()).unwrap()]),
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

    install_wallet(&create_result.canister_id, block_number).await?;
    Ok(create_result)
}

async fn install_wallet(canister_id: &Principal, block_number: u64) -> Result<(), String> {
    #[derive(CandidType, Deserialize, Clone, Debug)]
    pub struct Conf {
        pub origins: Vec<String>,
        pub management_canister: String,
    }

    let origins =  CONF.with(|c| c.borrow().clone().origins);
    let conf = Conf {
        origins,
        management_canister: id().to_string(),
    };
    let principal = caller();

    let arg = match candid::encode_args((principal, conf)) {
        Ok(a) => { a }
        Err(msg) => {
            return Err(format!(
                "An error happened during an arg encoding: {}: {}",
                principal.to_text(), msg
            ));
        }
    };

    //TODO maybe move to get_latest to avoid additional ICC or use ICQC
    let (versions, ): (Vec<String>, ) = match call::call(
        get_repo_canister_id(),
        "get_available_versions",
        (),
    ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };

    let sem_ver = versions.into_iter()
        .map(|v| Version::parse(&v).unwrap())
        .max().unwrap_or_else(|| trap("No semver versions found"));

    let (wasm, ): (VaultWasm, ) = match call::call(
        get_repo_canister_id(),
        "get_by_version",
        (sem_ver.to_string(), ),
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

    CANISTERS.with(|c| c.borrow_mut().push(VaultCanister {
        canister_id: canister_id.clone(),
        initiator: principal,
        block_number,
    }));

    Ok(())
}

#[query]
async fn canister_balance() -> u64 {
    ic_cdk::api::canister_balance()
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    canisters: Vec<VaultCanister>,
}

#[pre_upgrade]
pub fn stable_save() {
    let trs: Vec<VaultCanister> = CANISTERS.with(|trss| {
        trss.borrow().clone()
    });
    let mem = Memory {
        canisters: trs,
    };
    storage::stable_save((mem, )).unwrap();
}

#[post_upgrade]
pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    CANISTERS.with(|c| c.borrow_mut().extend(mo.canisters));
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

#[update]
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

pub fn get_repo_canister_id() -> Principal {
   CONF.with(|c| Principal::from_text(c.borrow().repo_canister_id.clone()).unwrap())
}

pub fn get_initial_cycles_balance() -> u128 {
    CONF.with(|c| c.borrow().initial_cycles_balance)
}

pub fn get_destination_address() -> String {
    CONF.with(|c| c.borrow().destination_address.clone())
}

pub fn get_payment_cycles() -> u64 {
    CONF.with(|c| c.borrow().payment_cycles)
}
