use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::collections::HashSet;
use ic_cdk::api::time;
use sha2::digest::typenum::private::IsEqualPrivate;
use crate::enums::{ObjectState, TransactionState};
use crate::transaction::quorum_transaction::QuorumTransaction;
use crate::transaction::transaction::{TransactionNew, TrType};
use candid::CandidType;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};
use crate::util::caller_to_address;
use crate::vault_service::VaultRole;

thread_local! {
    pub static MEMBERS: RefCell<Vec<Member>> = RefCell::new(Default::default());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Member {
    pub id: String,
    pub role: VaultRole,
    pub name: String,
    pub state: ObjectState,
    pub modified_date: u64,
    pub created_date: u64,
}

impl Member {
    pub fn new(id: String, role: VaultRole, name: String) -> Self {
        Member {
            id,
            role,
            name,
            state: ObjectState::Active,
            modified_date: time(),
            created_date: time(),
        }
    }
}

pub fn get_member_by_id(id: &String) -> Member {
    MEMBERS.with(|mrs| {
        match mrs.borrow().iter()
            .find(|x| x.id.eq(id)) {
            None => { trap("No such member") }
            Some(x) => { x.clone() }
        }
    })
}
pub fn store_member(member: Member) {
    MEMBERS.with(|mrs| {
        mrs.borrow_mut().push(member);
    })
}

pub fn get_members() -> Vec<Member> {
    MEMBERS.with(|mrs| {
        mrs.borrow().clone()
    })
}

pub fn restore_member(member: Member) {
    MEMBERS.with(|mrs| {
        let mut members = mrs.borrow_mut();
        members.retain(|existing| existing.id != member.id);
        members.push(member);
    });
}


