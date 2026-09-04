use std::cell::RefCell;
use std::collections::HashMap;

use api::call;
use candid::{export_service, Principal};
use candid::CandidType;
use ic_cdk::{api, call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::{CanisterIdRecord, CanisterInstallMode, CanisterSettings,
                                             delete_canister, InstallCodeArgument, stop_canister};
use ic_cdk_macros::*;
use ic_ledger_types::{GetBlocksArgs, MAINNET_LEDGER_CANISTER_ID, Operation, query_blocks};
pub use semver::Version;
use serde::{Deserialize, Serialize};

use nfid_certified::{CertifiedResponse, get_trusted_origins_cert, update_trusted_origins};

use crate::config::{Conf, CONF};
use crate::cost::create_canister_cost;
use crate::payment::LEDGER_FEE_E8S;
use crate::payment::{charge_user, manager_icp_balance, parse_destination, sweep_to, e8s_to_cycles, get_icp_xdr_rate, nat_to_u64, pay_protocol,
                     refund_user, split_payment, top_up_self};

mod config;
mod cost;
mod payment;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultCanister {
    canister_id: Principal,
    initiator: Principal,
    block_number: u64,
    vault_type: VaultType,
}

thread_local! {
    pub static CANISTERS: RefCell<Vec<VaultCanister>> = RefCell::new(Vec::default());
    /// Callers with a vault creation in flight, guarding against a second call
    /// slipping in while the first one is awaiting the ledger.
    static IN_FLIGHT: RefCell<HashMap<Principal, u64>> = RefCell::new(HashMap::default());
}

/// A lock is dropped after this timeout so that a trapping call cannot block a
/// caller forever: a trap only rolls the state back to the last await point.
const LOCK_TIMEOUT_NANOS: u64 = 5 * 60 * 1_000_000_000;

fn acquire_lock(caller: Principal) -> bool {
    IN_FLIGHT.with(|l| {
        let mut locks = l.borrow_mut();
        let now = api::time();
        match locks.get(&caller) {
            Some(started) if now - started < LOCK_TIMEOUT_NANOS => false,
            _ => {
                locks.insert(caller, now);
                true
            }
        }
    })
}

fn release_lock(caller: Principal) {
    IN_FLIGHT.with(|l| l.borrow_mut().remove(&caller));
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
    verify_payment(block_number).await;
    let initiator = owner.unwrap_or_else(|| caller());
    provision_vault(initiator, vault_type.unwrap_or_else(|| VaultType::Pro), block_number).await
}

/// Creates a vault paid with an ICRC-2 allowance instead of a pre-made ledger transfer.
///
/// The caller has to approve at least `icp_price + ledger fee` e8s for this canister
/// beforehand. The payment is pulled first, the vault is created from the cycles already
/// held by the manager, and only afterwards the payment is split between a CMC top up
/// that refills those cycles and the protocol wallet. Keeping the conversion last means
/// the ICP is still untouched on the manager account while the vault is being built, so
/// any failure along the way can be refunded in full.
#[update]
async fn create_canister_icrc2(vault_type: Option<VaultType>, owner: Option<Principal>) -> Result<CreateResult, String> {
    let payer = caller();
    if payer == Principal::anonymous() {
        return Err("Anonymous caller cannot create a vault".to_string());
    }
    if !acquire_lock(payer) {
        return Err("A vault creation is already in progress for this caller".to_string());
    }
    let result = create_canister_icrc2_inner(payer, vault_type, owner).await;
    release_lock(payer);
    result
}

async fn create_canister_icrc2_inner(payer: Principal, vault_type: Option<VaultType>, owner: Option<Principal>) -> Result<CreateResult, String> {
    let price = get_payment_cycles();
    let split = split_payment(price)?;
    let required_cycles = required_cycles()?;

    // Refuse before touching the user funds if the cycles part of the payment would not
    // refill what this vault costs to create.
    let rate = get_icp_xdr_rate().await?;
    let expected_cycles = e8s_to_cycles(split.to_cycles, rate);
    if expected_cycles < required_cycles {
        return Err(format!(
            "The configured price of {} e8s converts to {} cycles at the current rate, {} are required",
            price, expected_cycles, required_cycles
        ));
    }

    let block_index = charge_user(payer, price).await?;

    let initiator = owner.unwrap_or_else(|| payer);
    let created = provision_vault(
        initiator,
        vault_type.unwrap_or_else(|| VaultType::Pro),
        nat_to_u64(&block_index),
    ).await;

    let created = match created {
        Ok(x) => x,
        Err(reason) => {
            // The payment is still sitting on the manager account, give it back as a whole.
            return match refund_user(payer, price).await {
                Ok(_) => Err(reason),
                Err(refund_error) => Err(format!(
                    "{}. The refund failed as well: {}. The payment is held on the manager at block {}",
                    reason, refund_error, block_index
                )),
            };
        }
    };

    // The vault exists and belongs to the user from here on, so neither of the two
    // settlement transfers may fail the call. Any leftover stays on the manager account
    // and can be swept later.
    if let Err(e) = top_up_self(split.to_cycles).await {
        api::print(format!("Failed to convert {} e8s into cycles: {}", split.to_cycles, e));
    }
    if let Err(e) = pay_protocol(split.to_protocol).await {
        api::print(format!("Failed to send the protocol cut of {} e8s: {}", split.to_protocol, e));
    }

    Ok(created)
}

/// Guard for methods only the canister controllers may call.
fn is_caller_controller() -> Result<(), String> {
    if api::is_controller(&caller()) {
        Ok(())
    } else {
        Err("Only a controller can call this method".to_string())
    }
}

/// ICP currently held on the manager default account.
#[update(guard = "is_caller_controller")]
async fn get_icp_balance() -> Result<u64, String> {
    manager_icp_balance().await
}

/// Moves ICP off the manager account.
///
/// The manager is not meant to hold ICP: a payment is converted into cycles and
/// settled to the protocol wallet within the same call. Money left behind means a
/// step did not go through, most often a refund that the ledger rejected, and this
/// is how it is recovered.
///
/// `to` accepts an account identifier in hex or a principal, and defaults to the
/// configured protocol wallet. `amount_e8s` defaults to the whole balance minus the
/// transfer fee.
#[update(guard = "is_caller_controller")]
async fn sweep_icp(to: Option<String>, amount_e8s: Option<u64>) -> Result<u64, String> {
    let destination = match to {
        Some(ref text) => parse_destination(text)?,
        None => parse_destination(&CONF.with(|c| c.borrow().destination_address.clone()))?,
    };

    let balance = manager_icp_balance().await?;
    let amount = match amount_e8s {
        Some(requested) => requested,
        // The fee is taken on top of the amount, so the most that can leave the
        // account is the balance minus one fee.
        None => balance.checked_sub(LEDGER_FEE_E8S).unwrap_or(0),
    };

    if amount == 0 {
        return Err(format!("Nothing to sweep, the balance is {} e8s", balance));
    }
    if amount + LEDGER_FEE_E8S > balance {
        return Err(format!(
            "Cannot sweep {} e8s: the balance is {} e8s and the transfer fee is {} e8s",
            amount, balance, LEDGER_FEE_E8S
        ));
    }

    sweep_to(destination, amount).await
}

/// Cycles a new vault costs the manager: what the vault is funded with, plus what the
/// replica charges for creating the canister on this subnet.
fn required_cycles() -> Result<u128, String> {
    Ok(get_initial_cycles_balance() + create_canister_cost()?)
}

/// Everything the frontend needs to show a price and build the ICRC-2 approval.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CreationPrice {
    /// e8s charged from the user by `create_canister_icrc2`.
    pub price_e8s: u64,
    /// e8s the user has to approve for this canister: the price plus the ledger fee
    /// that `icrc2_transfer_from` takes on top of the transferred amount.
    pub approve_e8s: u64,
    /// Cycles the new vault is funded with.
    pub initial_cycles_balance: u128,
    /// Cycles the replica charges for creating the canister on this subnet.
    pub creation_fee_cycles: u128,
    /// Cycles a vault costs the manager in total.
    pub total_cycles: u128,
    /// Cycles the cycles part of the payment converts into at the current rate.
    pub payment_cycles: u128,
    /// ICP/XDR rate the conversion above is based on.
    pub xdr_permyriad_per_icp: u64,
    /// False when the current rate makes the configured price too low to cover a vault,
    /// in which case `create_canister_icrc2` refuses before charging anyone.
    pub covered: bool,
}

/// Current price of a vault. Not a query: the ICP/XDR rate comes from the CMC.
#[update]
async fn get_creation_price() -> Result<CreationPrice, String> {
    let price_e8s = get_payment_cycles();
    let split = split_payment(price_e8s)?;
    let initial_cycles_balance = get_initial_cycles_balance();
    let creation_fee_cycles = create_canister_cost()?;
    let total_cycles = initial_cycles_balance + creation_fee_cycles;

    let xdr_permyriad_per_icp = get_icp_xdr_rate().await?;
    let payment_cycles = e8s_to_cycles(split.to_cycles, xdr_permyriad_per_icp);

    Ok(CreationPrice {
        price_e8s,
        approve_e8s: price_e8s + LEDGER_FEE_E8S,
        initial_cycles_balance,
        creation_fee_cycles,
        total_cycles,
        payment_cycles,
        covered: payment_cycles >= total_cycles,
        xdr_permyriad_per_icp,
    })
}

/// Creates an empty canister, installs the latest vault wasm into it and hands the
/// control over to the vault itself. The canister is created through the management
/// canister on purpose: it places the vault on the same subnet as this manager, which
/// is what keeps production vaults on the fiduciary subnet.
async fn provision_vault(initiator: Principal, vault_type: VaultType, block_number: u64) -> Result<CreateResult, String> {
    let cycles = required_cycles()?;

    let set = CanisterSettings {
        controllers: Some(vec![id()]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
        reserved_cycles_limit: None,
    };

    let args = CreateCanisterArgs {
        cycles,
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

    if let Err(install_error) = install_wallet(&create_result.canister_id, &initiator).await {
        // Nothing was handed over yet, so the canister is still ours to delete and the
        // cycles locked in it come back to the manager.
        discard_canister(create_result.canister_id).await;
        return Err(install_error);
    }

    CANISTERS.with(|c| c.borrow_mut().push(VaultCanister {
        canister_id: create_result.canister_id.clone(),
        initiator,
        block_number,
        vault_type,
    }));

    let _ = update_settings(UpdateSettingsArg {
        canister_id: create_result.canister_id.clone(),
        settings: CanisterSettings {
            controllers: Some(vec![create_result.canister_id]),
            compute_allocation: None,
            freezing_threshold: None,
            memory_allocation: None,
            reserved_cycles_limit: None,
        },
    }).await;
    Ok(create_result)
}

/// Stops and deletes a canister that failed to receive its wasm, refunding its cycles
/// to the manager. Failures are only logged: the caller is already returning an error.
async fn discard_canister(canister_id: Principal) {
    let arg = CanisterIdRecord { canister_id };
    if let Err((code, msg)) = stop_canister(arg.clone()).await {
        api::print(format!("Failed to stop the canister {}: {}: {}", canister_id, code as u8, msg));
        return;
    }
    if let Err((code, msg)) = delete_canister(arg).await {
        api::print(format!("Failed to delete the canister {}: {}: {}", canister_id, code as u8, msg));
    }
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
        get_repo_canister_id()?,
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

    match call::call::<_, ()>(
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

fn get_repo_canister_id() -> Result<Principal, String> {
    let configured = CONF.with(|c| c.borrow().repo_canister_id.clone());
    Principal::from_text(&configured)
        .map_err(|e| format!("Invalid repo canister id {} in the config: {}", configured, e))
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