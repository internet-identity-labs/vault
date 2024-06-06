
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::hash::Hash;
use crate::enums::TransactionState::Executed;
use crate::get_state;
use crate::transaction::member::members::Member;
use crate::transaction::policy::policy::Policy;
use crate::transaction::vault::quorum::Quorum;
use crate::transaction::transaction::ITransaction;
use crate::transaction::transaction_service::get_all_transactions;
use crate::transaction::wallet::wallet::Wallet;

thread_local! {
    pub static STATE: RefCell<VaultState> = RefCell::new(VaultState::default());
    pub static ICRC1_STORAGE: RefCell<Vec<ICRC1>> = RefCell::new(Default::default());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, Eq)]
pub struct ICRC1 {
    pub ledger: Principal,
    pub index: Option<Principal>,
}

impl PartialEq for ICRC1 {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

impl Hash for ICRC1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VaultState {
    pub quorum: Quorum,
    pub wallets: Vec<Wallet>,
    pub members: Vec<Member>,
    pub policies: Vec<Policy>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icrc1_canisters: Vec<ICRC1>,
}

impl VaultState {
   pub fn default() -> Self {
        VaultState {
            quorum: Quorum::default(),
            wallets: vec![],
            members: vec![],
            policies: vec![],
            name: None,
            description: None,
            icrc1_canisters: vec![]
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
    let transactions = get_all_transactions();
    let state = define_state(transactions, tr_id).await;
    state
}

pub async fn define_state(transactions: Vec<Box<dyn ITransaction>>, tr_id: Option<u64>) -> VaultState {
    let mut state = VaultState::default();
    let mut sorted_trs = transactions
        .into_iter()
        .filter(|transaction| {
            let is_vault_state = transaction.is_vault_state();
            let is_executed = transaction.get_state() == &Executed;
            match tr_id {
                None => is_vault_state && is_executed,
                Some(tr) => is_vault_state && is_executed && transaction.get_id() <= tr,
            }
        })
        .collect::<Vec<Box<dyn ITransaction>>>();

    sorted_trs.sort_by(|a,b| -> std::cmp::Ordering {
        b.get_id().cmp(&a.get_id())
    });

    while let Some(mut trs) = sorted_trs.pop() {
        state = trs.execute(state).await;
    }
    state.icrc1_canisters = get_icrc1_canisters();
    state
}

pub async fn save_icrc1_canister(ledger: Principal, index: Option<Principal>) -> VaultState {
    ICRC1_STORAGE.with(|c| {
        c.borrow_mut().push(ICRC1 {
            ledger,
            index
        });
    });
    get_state(None).await
}

pub async fn delete_icrc1_canister(ledger: Principal) -> VaultState {
    ICRC1_STORAGE.with(|c| {
        let mut canisters_borrowed = c.borrow_mut();
        canisters_borrowed.retain(|c| c.ledger != ledger);
    });
    get_state(None).await
}

pub fn get_icrc1_canisters() -> Vec<ICRC1> {
    ICRC1_STORAGE.with(|c| c.borrow().clone())
}