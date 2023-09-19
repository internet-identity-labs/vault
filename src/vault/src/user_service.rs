use std::collections::HashSet;
use candid::CandidType;
use ic_cdk::trap;
use serde::Deserialize;
use crate::memory::USERS;
use serde::Serialize;

use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct User {
    pub address: String,
    pub vaults: HashSet<u64>,
}

pub fn get_or_new_by_address(address: String) -> User {
    USERS.with(|users| {
        let mut borrowed = users.borrow_mut();
        match borrowed.get_mut(&address) {
            None => {
                let new_user = User { address: address.clone(), vaults: hashset!{} };
                borrowed.insert(address, new_user.clone());
                new_user
            }
            Some(user) => {
                user.clone()
            }
        }
    })
}

pub fn migrate_to_address(from_address : String, to_address: String) -> bool {
    USERS.with(|users| {
        let mut borrowed = users.borrow_mut();
        match borrowed.get_mut(&from_address) {
            None => {
                trap("Should not be the case")
            }
            Some(user) => {
                let new_user = User { address: to_address.clone(), vaults: user.vaults.clone()};
                borrowed.insert(to_address, new_user);
            }
        }
        match borrowed.remove(&from_address) {
            None => {false}
            Some(_) => {true}
        }
    })
}

pub fn get_or_new_by_caller() -> User {
    let address = caller_to_address();
    get_or_new_by_address(address)
}

pub fn restore(user: User) -> Option<User> {
    USERS.with(|users| {
        users.borrow_mut().insert(user.address.clone(), user)
    })
}

