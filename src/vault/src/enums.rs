use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum TransactionState {
    Approved,
    Rejected,
    Pending,
    Canceled,
    Blocked,
    Executed
}


impl ToString for TransactionState {
    fn to_string(&self) -> String {
        match self {
            TransactionState::Approved => {"A".to_string()}
            TransactionState::Rejected => {"R".to_string()}
            TransactionState::Pending => {"P".to_string()}
            TransactionState::Canceled => {"C".to_string()}
            TransactionState::Blocked => {"B".to_string()}
            TransactionState::Executed => {"E".to_string()}
        }
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ObjectState {
    Archived,
    Active
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Action {
    Create,
    Archive,
    Update
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Network {
    IC,
    BTC,
    ETH
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Currency {
    ICP,
}

