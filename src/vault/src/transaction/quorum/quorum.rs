use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use crate::transaction::transaction::{TransactionNew, TrType};

thread_local! {
    pub static QUORUM: RefCell<Quorum> = RefCell::new(Quorum::default());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Quorum {
    pub quorum: u8,
    pub modified_date: u64, //todo
}

impl Quorum {
    fn default() -> Self {
        Quorum {
            quorum: 1,
            modified_date: time(),
        }
    }
}

pub fn get_quorum() -> Quorum {
    QUORUM.with(|qp| {
        qp.borrow().clone()
    })
}

pub fn update_quorum(mut q: Quorum) {
    QUORUM.with(|qp| {
        qp.replace(q);
    })
}

pub fn get_vault_admin_block_predicate(tr: &Box<dyn TransactionNew>) -> bool {
    return match tr.clone_self().get_type() {
        TrType::Quorum => true,
        TrType::MemberArchive => true,
        TrType::MemberUnarchive => true,
        TrType::MemberCreate => true,
        TrType::MemberUpdateName => true,
        TrType::MemberUpdateRole => true,
        TrType::WalletCreate => true,
        TrType::WalletUpdateName => true,
        _ => false,
    };
}

