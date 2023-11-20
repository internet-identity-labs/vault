// use async_trait::async_trait;
// use candid::CandidType;
// use ic_ledger_types::BlockIndex;
// use serde::{Deserialize, Serialize};
//
// use crate::enums::{Currency, TransactionState};
// use crate::impl_basic_for_transaction;
// use crate::state::VaultState;
// use crate::transaction::basic_transaction::BasicTransaction;
// use crate::transaction::basic_transaction::BasicTransactionFields;
// use crate::transaction::policy::policy::Policy;
// use crate::transaction::policy::policy_create_transaction::PolicyCreateTransaction;
// use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
// use crate::transaction::transaction_builder::TransactionBuilder;
//
// impl_basic_for_transaction!(TransferTransaction);
// #[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
// pub struct TransferTransaction {
//     pub common: BasicTransactionFields,
//     policy: Option<String>,
//     wallet: String,
//     block_index: Option<BlockIndex>,
//     amount: u64,
//     currency: Currency,
//     address: String,
// }
//
// impl TransferTransaction {
//     fn new(address: String, currency: Currency,
//            wallet: String, amount: u64, state: TransactionState) -> Self {
//         TransferTransaction {
//             common: BasicTransactionFields::new(state, TrType::Transfer, false),
//             wallet,
//             policy: None,
//             currency,
//             block_index: None,
//             amount,
//             address,
//         }
//     }
// }
//
// #[async_trait]
// impl TransactionNew for TransferTransaction {
//     async fn execute(&self, mut state: VaultState) -> VaultState {
//         //todo transfer
//         state
//     }
//
//     fn to_candid(&self) -> TransactionCandid {
//         let trs: TransferTransaction = self.clone();
//         TransactionCandid::TransferTransaction(trs)
//     }
//
//     fn define_state(&mut self) {
//         if !is_blocked(|tr| {
//             return get_vault_state_block_predicate(tr);
//         }) {
//             if self.policy <= self.get_common_ref().approves.len() as u8 {
//                 self.set_state(Approved)
//             } else {
//                 self.set_state(Pending)
//             }
//         }
//     }
// }
//
//
// pub struct TransferTransactionBuilder {
//     wallet: String,
//     amount: u64,
//     currency: Currency,
//     address: String,
// }
//
// impl TransferTransactionBuilder {
//     pub fn init(uid: String, currency: Currency, amount_threshold: u64,
//                 member_threshold: u8, wallets: Vec<String>) -> Self {
//
//         return TransferTransactionBuilder {
//             uid,
//             currency,
//             amount_threshold,
//             member_threshold,
//             wallets,
//         };
//     }
// }
//
// impl TransactionBuilder for TransferTransactionBuilder {
//
//     fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
//         let trs = PolicyCreateTransaction::new(
//             self.uid.clone(),
//             self.currency.clone(),
//             self.amount_threshold.clone(),
//             self.member_threshold.clone(),
//             self.wallets.clone(),
//             state,
//         );
//         Box::new(trs)
//     }
//
//     fn get_block_predicate(&mut self, tr: &Box<dyn TransactionNew>) -> bool {
//         return get_vault_state_block_predicate(tr);
//     }
//
//     fn define_initial_state(&mut self) -> TransactionState {
//         if is_blocked(|tr| {
//             self.get_block_predicate(tr)
//         }) { Blocked } else { Pending }
//     }
//
// }
//
//
