
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use crate::transaction::member::members::Member;
use crate::transaction::policy::policy::Policy;
use crate::transaction::quorum::quorum::Quorum;
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