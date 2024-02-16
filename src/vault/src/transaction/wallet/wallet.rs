use candid::{CandidType, Principal};
use ic_cdk::{call, trap};
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{Network};
use crate::state::VaultState;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Wallet {
    pub uid: String,
    pub name: String,
    pub network: Network,
    pub modified_date: u64,
    pub created_date: u64,
}

impl Wallet {
    pub fn new(uid: String, name: String, network: Network) -> Self {
        Wallet {
            uid,
            name,
            network,
            modified_date: time(),
            created_date: time(),
        }
    }
}

pub fn restore_wallet(wallet: Wallet, mut state: VaultState) -> VaultState {
    state.wallets.retain(|existing| existing.uid != wallet.uid);
    state.wallets.push(wallet);
    state
}

