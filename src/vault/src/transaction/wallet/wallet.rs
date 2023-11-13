use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_cdk::{call, trap};
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState;
use crate::policy_service::Currency;

thread_local! {
    pub static WALLETS: RefCell<Vec<Wallet>> = RefCell::new(Default::default());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Wallet {
    pub uid: String,
    pub name: String,
    pub currency: Currency,
    pub state: ObjectState,
    pub modified_date: u64,
    pub created_date: u64,
}

impl Wallet {
    pub fn new(uid: String, name: String) -> Self {
        Wallet {
            uid,
            name,
            currency: Currency::ICP,
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        }
    }
}

pub fn get_wallet_by_id(uid: &String) -> Wallet {
    WALLETS.with(|mrs| {
        match mrs.borrow().iter()
            .find(|x| x.uid.eq(uid)) {
            None => { trap("No such wallet") }
            Some(x) => { x.clone() }
        }
    })
}

pub fn store_wallet(wallet: Wallet) {
    WALLETS.with(|mrs| {
        mrs.borrow_mut().push(wallet);
    })
}

pub fn get_wallets() -> Vec<Wallet> {
    WALLETS.with(|mrs| {
        mrs.borrow().clone()
    })
}

pub fn restore_wallet(member: Wallet) {
    WALLETS.with(|wts| {
        let mut wallets = wts.borrow_mut();
        wallets.retain(|existing| existing.uid != member.uid);
        wallets.push(member);
    });
}


pub async fn generate_address() -> String {
    let raw_rand: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get sub: {}", err)),
    };
    hex::encode(raw_rand)
}

