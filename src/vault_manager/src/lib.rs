use std::cell::RefCell;
use std::str::FromStr;

use candid::{export_service, Principal};
use candid::CandidType;
use ic_cdk::{api, caller, id};
use ic_cdk::api::management_canister::main::{CanisterInstallMode, CanisterSettings, InstallCodeArgument};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

const FEE: u128 = 100_000_000_000;
//0.1T - to be discussed
const INITIAL_CYCLES_BALANCE: u128 = 104_000_000_000;
pub const WASM: &[u8] = include_bytes!("vault.wasm");

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultCanister {
    canister_id: Principal,
    initiator: Principal,
}

thread_local! {
    pub static CANISTERS: RefCell<Vec<VaultCanister>> = RefCell::new(Vec::default());
}

#[derive(CandidType, Clone, Deserialize)]
struct CreateCanisterArgs<TCycles> {
    cycles: TCycles,
    settings: CanisterSettings,
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

#[update]
async fn get_all_canisters() -> Vec<VaultCanister> {
    CANISTERS.with(|c| c.borrow().clone())
}

#[update]
async fn create_canister_call() -> Result<CreateResult, String> {
    let set = CanisterSettings {
        //add this canister as a controller + for now add a debug dude
        controllers: Some(vec![id(), Principal::from_text("lh6kg-7ebfk-bwa26-zgl6l-l27vx-xnnr4-ow2n4-mm4cq-tfjky-rs5gq-5ae".to_string()).unwrap()]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };

    let args = CreateCanisterArgs {
        cycles: INITIAL_CYCLES_BALANCE + FEE,
        settings: set.clone(),
    };

    #[derive(CandidType)]
    struct In {
        settings: Option<CanisterSettings>,
    }

    let in_arg = In {
        settings: Some(set),
    };

    let (create_result, ): (CreateResult, ) = match api::call::call_with_payment128(
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

    install_wallet(&create_result.canister_id).await?;
    Ok(create_result)
}

async fn install_wallet(canister_id: &Principal) -> Result<(), String> {
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

    #[derive(CandidType, Deserialize, Clone, Debug)]
    pub struct Conf {
        pub origins: Vec<String>,
    }

    //TODO: handle logic with origins depends on the environment
    let conf = Conf {
        origins: vec![],
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

    let arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: WASM.to_vec(),
        arg,
    };

    match api::call::call(
        Principal::management_canister(),
        "install_code",
        (arg, ), ).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };
    CANISTERS.with(|c| c.borrow_mut().push(VaultCanister {
        canister_id: canister_id.clone(),
        initiator: principal,
    }));
    Ok(())
}

#[query]
async fn canister_balance() -> u64 {
    ic_cdk::api::canister_balance()
}

#[test]
fn sub_account_test() {}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}