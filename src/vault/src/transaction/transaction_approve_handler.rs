use ic_cdk::api::time;
use crate::enums::TransactionState;
use crate::transaction::transaction::TransactionCandid;
use crate::transaction::transactions_service::{get_by_id, restore_transaction};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;

pub fn handle_approve(tr_id: u64, state: TransactionState) -> TransactionCandid {
    let mut trs = get_by_id(tr_id);
    let approve = Approve {
        signer: caller_to_address(),
        created_date: time(),
        status: state,
    };
    trs.handle_approve(approve);
    trs.define_state();
    restore_transaction(trs.clone());
    trs.to_candid()
}
