use candid::CandidType;
use ic_cdk::api::time;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};

use crate::enums::{VaultRole};
use crate::state::{STATE, VaultState};
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub member_id: String,
    pub role: VaultRole,
    pub name: String,
    pub modified_date: u64,
    pub created_date: u64,
}

impl Member {
    pub fn new(id: String, role: VaultRole, name: String) -> Self {
        Member {
            member_id: id,
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
            .find(|m| m.member_id.eq(&caller))
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


