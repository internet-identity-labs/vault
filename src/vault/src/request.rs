use crate::enums::TransactionState;

// use candid::Principal;
//
use crate::enums::ObjectState;
use candid::{CandidType, Deserialize, Principal};
//
// #[derive(CandidType, Deserialize, Clone)]
// pub struct TransactionRegisterRequest {
//     pub amount: u64,
//     pub address: String,
//     pub wallet_id: String,
// }
//
// #[derive(CandidType, Deserialize, Clone)]
// pub struct VaultRegisterRequest {
//     pub name: String,
//     pub description: Option<String>,
// }
//
// #[derive(CandidType, Deserialize, Clone)]
// pub struct VaultMemberRequest {
//     pub address: String,
//     pub name: Option<String>,
//     pub role: VaultRole,
//     pub state: ObjectState
// }
//
// #[derive(CandidType, Deserialize, Clone)]
// pub struct WalletRegisterRequest {
//     pub name: Option<String>,
// }
//
#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionApproveRequest {
    pub transaction_id: u64,
    pub state: TransactionState,
}
//
// #[derive(CandidType, Deserialize, Clone)]
// pub struct PolicyRegisterRequest {
//    pub policy_type: PolicyType,
// }
//
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}