
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use crate::enums::TransactionState::Executed;
use crate::transaction::member::members::Member;
use crate::transaction::policy::policy::Policy;
use crate::transaction::quorum::quorum::Quorum;
use crate::transaction::transaction::ITransaction;
use crate::transaction::transactions_service::get_all_transactions;
use crate::transaction::wallet::wallet::Wallet;

thread_local! {
    pub static STATE: RefCell<VaultState> = RefCell::new(VaultState::default());
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultState {
    pub quorum: Quorum,
    pub wallets: Vec<Wallet>,
    pub members: Vec<Member>,
    pub policies: Vec<Policy>
}

impl VaultState {
   pub fn default() -> Self {
        VaultState {
            quorum: Quorum::default(),
            wallets: vec![],
            members: vec![],
            policies: vec![],
        }
    }
}

pub fn get_current_state() -> VaultState {
    let state = STATE.with(|st| st.borrow().clone());
    state
}

pub fn restore_state(state: VaultState) {
    STATE.with(|st| st.replace(state));
}


pub async fn get_vault_state(tr_id: Option<u64>) -> VaultState {
    let mut state = VaultState::default();
    let mut transactions = get_all_transactions();
    transactions.sort();
    let mut sorted_trs = transactions
        .into_iter()
        .filter(|transaction| {
            let is_vault_state = transaction.is_vault_state();
            let is_executed = transaction.get_state().eq(&Executed);
            match tr_id {
                None => is_vault_state && is_executed,
                Some(tr) => is_vault_state && is_executed && transaction.get_id() <= tr,
            }
        })
        .collect::<Vec<Box<dyn ITransaction>>>();

    while let Some(mut trs) = sorted_trs.pop() {        //TODO
        state = trs.execute(state).await;
    }
    state
}