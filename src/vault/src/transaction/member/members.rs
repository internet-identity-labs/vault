use candid::CandidType;
use ic_cdk::api::time;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState;
use crate::state::{STATE, VaultState};
use crate::vault_service::VaultRole;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub member_id: String,
    pub role: VaultRole,
    pub name: String,
    pub state: ObjectState,
    pub modified_date: u64,
    pub created_date: u64,
}

impl Member {
    pub fn new(id: String, role: VaultRole, name: String) -> Self {
        Member {
            member_id: id,
            role,
            name,
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        }
    }
}


pub fn get_members() -> Vec<Member> {
    STATE.with(|mrs| {
        mrs.borrow().members.clone()
    })
}

pub fn restore_member(member: Member, mut state: VaultState) -> VaultState {
        state.members.retain(|existing| existing.member_id != member.member_id);
        state.members.push(member);
        state
}


