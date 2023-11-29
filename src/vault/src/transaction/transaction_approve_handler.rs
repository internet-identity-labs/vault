use ic_cdk::api::time;
use ic_cdk::trap;

use crate::enums::TransactionState;
use crate::enums::TransactionState::Approved;
use crate::enums::TransactionState::Executed;
use crate::enums::TransactionState::Rejected;
use crate::security_service::verify_caller;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transactions_service::{get_by_id, restore_transaction};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

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
