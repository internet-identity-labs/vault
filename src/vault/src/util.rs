use std::convert::TryInto;
use ic_cdk::{caller, trap};
use ic_ledger_types::{AccountIdentifier, Subaccount};

pub fn caller_to_address() -> String {
    return AccountIdentifier::new(&caller(), &Subaccount([1; 32])).to_string();
}


pub fn to_array<T>(v: Vec<T>) -> [T; 32] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| trap(&format!("Expected a Vec of length {} but it was {}", 32, v.len())))
}