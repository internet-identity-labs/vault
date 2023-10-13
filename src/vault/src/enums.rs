use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum TransactionState {
    Approved,
    Rejected,
    Pending,
    Canceled
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ObjectState {
    Archived,
    Active
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Backup {
    Wallets,
    Users,
    Policies,
    Transactions
}


