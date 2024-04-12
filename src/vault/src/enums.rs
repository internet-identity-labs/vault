use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum TransactionState {
    Approved,
    Rejected,
    Pending,
    Blocked,
    Executed,
    Purged,
    Failed,
}


impl ToString for TransactionState {
    fn to_string(&self) -> String {
        match self {
            TransactionState::Approved => {"Approved".to_string()}
            TransactionState::Rejected => {"Rejected".to_string()}
            TransactionState::Pending => {"Pending".to_string()}
            TransactionState::Blocked => {"Blocked".to_string()}
            TransactionState::Executed => {"Executed".to_string()}
            TransactionState::Purged => {"Purged".to_string()}
            TransactionState::Failed => {"Failed".to_string()}
        }
    }
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

#[derive(Clone, Debug, CandidType, Deserialize, Copy, Eq, PartialEq, Serialize)]
pub enum VaultRole {
    Admin,
    Member,
}

