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

