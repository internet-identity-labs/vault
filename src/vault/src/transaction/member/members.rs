use candid::{CandidType, Principal};
use ic_cdk::api::time;
use ic_cdk::trap;
use ic_ledger_types::{AccountIdentifier, DEFAULT_SUBACCOUNT};
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::{Deserialize, Serialize};

use crate::enums::VaultRole;
use crate::state::{STATE, VaultState};
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub member_id: String,
    pub role: VaultRole,
    pub name: String,
    pub modified_date: u64,
    pub created_date: u64,
    pub account: Option<Account>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

impl Member {
    pub fn new(account: Account, role: VaultRole, name: String) -> Self {
        Member {
            member_id: calculate_id(account.clone()),
            account: Some(account),
            role,
            name,
            modified_date: time(),
            created_date: time(),
        }
    }
}

pub fn get_caller_role() -> VaultRole {
    let caller = caller_to_address();
    let role = STATE.with(|mrs| {
        mrs.borrow().members.iter()
            .find(|m| m.member_id.eq_ignore_ascii_case(&caller))
            .map(|m| m.role)
    });
    match role {
        None => {
            trap("Not registered");
        }
        Some(role) => {
            role
        }
    }
}

pub fn restore_member(member: Member, mut state: VaultState) -> VaultState {
    state.members.retain(|existing| existing.member_id != member.member_id);
    state.members.push(member);
    state
}

pub fn calculate_id(acc: Account) -> String {
    let member_id = AccountIdentifier::new(
        &acc.owner,
        &match acc.subaccount.clone() {
            None => { DEFAULT_SUBACCOUNT }
            Some(x) => { ic_ledger_types::Subaccount(x) }
        }).to_hex();
    member_id
}


