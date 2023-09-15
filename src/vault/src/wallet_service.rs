use std::collections::HashSet;

use candid::{CandidType, Principal};
use ic_cdk::{call, trap};
use serde::Deserialize;
use serde::{Serialize};

use crate::enums::ObjectState;
use crate::memory::WALLETS;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Wallet {
    pub uid: String,
    pub name: Option<String>,
    pub vaults: HashSet<u64>,
    pub state: ObjectState,
    pub created_date: u64,
    pub modified_date: u64,
}

pub fn new_and_store(name: Option<String>, vault_id: u64, address: String) -> Wallet {
    WALLETS.with(|wts| {
        let mut wallets = wts.borrow_mut();
        if wallets.contains_key(&address) {
            trap("Bad luck. Please retry")
        }
        let wallet_new = Wallet {
            uid: address.clone(),
            name,
            vaults: hashset![vault_id],
            state: ObjectState::Active,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
        };
        wallets.insert(address, wallet_new.clone());
        wallet_new
    })
}

pub fn restore(mut wallet: Wallet) -> Wallet {
    return WALLETS.with(|wallets| {
        wallet.modified_date = ic_cdk::api::time();
        wallets.borrow_mut().insert(wallet.uid.clone(), wallet.clone());
        wallet
    });
}

pub fn update(wallet: Wallet) -> Wallet {
    let mut old = get_by_uid(&wallet.uid);
    old.name = wallet.name;
    old.state = wallet.state;
    restore(old.clone())
}

pub fn get_by_uid(uid: &String) -> Wallet {
    WALLETS.with(|wallets| {
        match wallets.borrow().get(uid) {
            None => {
                trap("Not registered")
            }
            Some(wallet) => {
                wallet.clone()
            }
        }
    })
}

pub fn get_wallets(uids: HashSet<String>) -> Vec<Wallet> {
    WALLETS.with(|wallets| {
        let mut result: Vec<Wallet> = Default::default();
        for key in uids {
            match wallets.borrow().get(&key) {
                None => {
                    trap("Not registered")
                }
                Some(wallet) => {
                    result.push(wallet.clone())
                }
            }
        }
        result
    })
}

pub async fn generate_address() -> String {
    let raw_rand: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get sub: {}", err)),
    };
    hex::encode(raw_rand)
}



