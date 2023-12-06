use std::hash::{Hash, Hasher};
use ic_cdk::api::time;
use ic_cdk::trap;

use crate::enums::TransactionState;
use crate::enums::TransactionState::Approved;
use crate::enums::TransactionState::Executed;
use crate::enums::TransactionState::Rejected;
use crate::security_service::verify_caller;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transaction_service::{get_by_id, restore_transaction};
use crate::util::caller_to_address;
use serde::{Deserialize, Serialize};
use candid::CandidType;


#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionApproveRequest {
    pub transaction_id: u64,
    pub state: TransactionState,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Approve {
    pub signer: String,
    pub created_date: u64,
    pub status: TransactionState,
}

impl PartialEq for Approve {
    fn eq(&self, other: &Self) -> bool {
        self.signer.eq(&other.signer)
    }
}

impl Hash for Approve {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signer.hash(state)
    }
}

pub fn handle_approve(tr_id: u64, state: TransactionState) -> TransactionCandid {
    let mut trs = get_by_id(tr_id);

    match trs.get_state() {
        Rejected | Executed => {
            trap("Transaction is immutable")
        }
        _ => {}
    }

    match state {
        Approved | Rejected => {
            verify_caller(trs.get_accepted_roles());
            let approve = Approve {
                signer: caller_to_address(),
                created_date: time(),
                status: state,
            };
            trs.handle_approve(approve);
            restore_transaction(trs.clone());
            trs.to_candid()
        }
        _ => trap("Unexpected value"),
    }
}
