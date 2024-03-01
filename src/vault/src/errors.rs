use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum VaultError {
    MemberAlreadyExists,
    ThresholdAlreadyExists,
    MemberNotExists,
    WalletNotExists,
    PolicyNotExists,
    UIDAlreadyExists,
    CouldNotDefinePolicy,
    CanisterReject { message: String },
    QuorumNotReachable,
    ThresholdDefineError { message: String },
    ControllersUpdateError { message: String },
}