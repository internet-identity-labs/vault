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
}

pub fn get_or_new_by_address(address: String) -> User {
    USERS.with(|users| {
        let mut borrowed = users.borrow_mut();
        match borrowed.get_mut(&address) {
            None => {
                let new_user = User { address: address.clone() };
                borrowed.insert(address, new_user.clone());
                new_user
            }
            Some(user) => {
                user.clone()
            }
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

