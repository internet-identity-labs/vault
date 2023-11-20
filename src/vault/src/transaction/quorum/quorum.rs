use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::state::STATE;
use crate::transaction::transaction::{ITransaction, TrType};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Quorum {
    pub quorum: u8,
    pub modified_date: u64,
}

impl Quorum {
    pub fn default() -> Self {
        Quorum {
            quorum: 1,
            modified_date: time(),
        }
    }
}

pub fn get_quorum() -> Quorum {
    STATE.with(|qp| {
        qp.borrow().quorum.clone()
    })
}

