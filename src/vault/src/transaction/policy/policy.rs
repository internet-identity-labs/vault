use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::{Currency, ObjectState};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Policy {
    pub uid: String,
    pub amount_threshold: u64,
    pub member_threshold: u8,
    pub currency: Currency,
    pub wallets: Vec<String>,
    pub created_date: u64,
    pub modified_date: u64,
}


impl Policy {
    pub fn new(uid: String, currency: Currency, amount_threshold: u64, member_threshold: u8, wallets: Vec<String>) -> Self {
        Policy {
            uid,
            amount_threshold,
            member_threshold,
            currency,
            modified_date: time(),
            created_date: time(),
            wallets,
        }
    }
}