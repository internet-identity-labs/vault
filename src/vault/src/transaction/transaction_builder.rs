use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::collections::HashSet;

use candid::CandidType;
use ic_cdk::api::time;
use ic_cdk::print;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::enums::{Network, TransactionState};
use crate::transaction::member_transaction::{MemberTransaction, MemberTransactionBuilder};
use crate::transaction::members::{get_member_by_id, Member};
use crate::transaction::quorum::Quorum;
use crate::transaction::quorum_transaction;
use crate::transaction::quorum_transaction::{QuorumTransaction, QuorumTransactionBuilder};
use crate::transaction::transaction::TransactionNew;
use crate::transaction::transaction_request_handler::handle_transaction_request;
use crate::transaction::transactions_service::{get_all_transactions, get_by_id, get_unfinished_transactions, restore_transaction, store_transaction, TRANSACTIONS};
use crate::transaction_service::Approve;
use crate::util::caller_to_address;
use crate::vault_service::VaultRole;

pub trait TransactionBuilder {
    // type OutputType;
    fn define_initial_state(&mut self);
    fn define_initial_policy(&mut self);
    fn build(self) -> Box<dyn TransactionNew>;
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletCreateTransaction {
    pub network: Network,
    pub name: String,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TransferTransaction {
    pub wallet: String,
    pub to: String,
    pub amount: u64,
    pub block_index: Option<BlockIndex>,
}
