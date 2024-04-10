use std::cell::RefCell;

use candid::{CandidType, Principal};
use candid::export_service;
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk_macros::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use nfid_certified::{CertifiedResponse, get_trusted_origins_cert, update_trusted_origins};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultWasm {
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
    version: String,
    hash: String,
    description: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub origins: Vec<String>,
    pub controllers: Vec<Principal>,
}

thread_local! {
    pub static VAULT_VERSIONS: RefCell<Vec<VaultWasm >> = RefCell::new(Vec::default());
    pub static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        controllers: Default::default(),
        origins: Default::default(),
    });
}

#[init]
async fn init(conf: Conf) {
    CONFIG.with(|c| c.replace(conf));
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[update]
async fn sync_controllers() -> Vec<String> {
    let res: CallResult<(CanisterStatusResponse, )> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest {
            canister_id: id(),
        }, ),
    ).await;
    let controllers = res.unwrap().0.settings.controllers;
    CONFIG.with(|c| c.borrow_mut().controllers = controllers.clone());
    controllers.iter().map(|x| x.to_text()).collect()
}

#[derive(CandidType, Deserialize)]
struct WalletStoreWASMArgs {
    #[serde(with = "serde_bytes")]
    wasm_module: Vec<u8>,
    version: String,
    hash: String,
    description: Option<String>,
}

#[derive(CandidType, Deserialize)]
struct VersionWrapper {
    version: String,
    description: Option<String>,
}

#[update]
async fn get_available_versions() -> Vec<VersionWrapper> {
    VAULT_VERSIONS.with(|vv|
        vv.borrow().iter()
            .map(|vw| {
                VersionWrapper {
                    version: vw.version.clone(),
                    description: vw.description.clone(),
                }
            })
            .collect()
    )
}

#[query]
async fn get_latest_version() -> VaultWasm {
    let latest = VAULT_VERSIONS.with(|vv|
        vv.borrow().iter()
            .map(|vw| vw.version.clone())
            .map(|v| Version::parse(&v).unwrap())
            .max()
            .unwrap_or_else(|| trap("No semver versions found")));

    VAULT_VERSIONS.with(|vv|
        vv.borrow().iter()
            .find(|vw| vw.version == latest.to_string())
            .map(|vw| vw.clone())
            .unwrap_or_else(|| trap("Version not found"))
    )
}

#[query]
async fn get_by_version(version: String) -> VaultWasm {
    VAULT_VERSIONS.with(|vv|
        vv.borrow().iter()
            .find(|vw| vw.version == version)
            .map(|vw| vw.clone())
            .unwrap_or_else(|| trap("Version not found"))
    )
}

#[update]
async fn delete_by_version(version: String) {
    VAULT_VERSIONS.with(|vv| {
        let mut versions = vv.borrow_mut();
        let index = versions.iter().position(|v| v.version == version);
        match index {
            Some(i) => {
                versions.remove(i);
            }
            None => {
                trap("Version not found");
            }
        }
    });
}

#[update]
async fn add_version(args: WalletStoreWASMArgs) {
    trap_if_not_authenticated_admin();
    let sha256 = get_wasm_hash(args.wasm_module.clone());
    if sha256 != args.hash {
        trap(format!("Hashes do not match {} {}", sha256, args.hash).as_str());
    }
    VAULT_VERSIONS.with(|vv| {
        vv.borrow_mut().push(VaultWasm {
            wasm_module: args.wasm_module,
            version: args.version,
            hash: args.hash,
            description: args.description,
        })
    });
}

#[query]
async fn canister_balance() -> u64 {
    ic_cdk::api::canister_balance()
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    versions: Vec<VaultWasm>,
    config: Option<Conf>
}

#[pre_upgrade]
pub fn stable_save() {
    let trs: Vec<VaultWasm> = VAULT_VERSIONS.with(|vv| {
        vv.borrow().clone()
    });
    let conf: Conf = CONFIG.with(|vv| {
        vv.borrow().clone()
    });

    let mem = Memory {
        versions: trs,
        config: Some(conf),
    };
    storage::stable_save((mem, )).unwrap();
}

#[post_upgrade]
pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    VAULT_VERSIONS.with(|vv| vv.borrow_mut().extend(mo.versions.clone()));
    match mo.config {
        None => {}
        Some(conf) => {
            update_trusted_origins(conf.origins.clone());
            CONFIG.with(|vv| {
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

fn get_wasm_hash(wasm: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wasm);
    let result = hasher.finalize();

    return format!("0x{:x}", result);
}

fn trap_if_not_authenticated_admin() {
    let princ = caller();
    let controllers = CONFIG.with(|c| c.borrow_mut().controllers.clone());
    if !controllers.contains(&princ) {
        trap(format!("Unauthorised {}", princ).as_str());
    }
}

#[update]
async fn get_trusted_origins() -> Vec<String> {
    CONFIG.with(|c| c.borrow().clone().origins)
}

#[update]
async fn get_config() -> Conf {
    CONFIG.with(|c| c.borrow().clone())
}


#[query]
async fn get_trusted_origins_certified() -> CertifiedResponse {
    get_trusted_origins_cert()
}