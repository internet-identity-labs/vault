// use std::borrow::Borrow;
// use ic_cdk::trap;
// 
// use crate::{get_or_new_by_caller, PolicyType, vault_service, VaultRole};
// use crate::enums::ObjectState::Archived;
// 
// pub fn trap_if_not_permitted(accepted_roles: Vec<VaultRole>) {
//     let caller = get_or_new_by_caller();
//     // let vault = vault_service::get_by_id(&vault_id);
//     // let caller_member = vault.members
//     //     .iter()
//     //     .filter(|m| !m.state.eq(&Archived))
//     //     .find(|p| caller.address.eq(&p.user_uuid));
//     // match caller_member {
//     //     None => {
//     //         trap("Unauthorised")
//     //     }
//     //     Some(vault_member) => {
//     //         if !accepted_roles.is_empty() && !accepted_roles.contains(&vault_member.role) {
//     //             trap("Not enough permissions")
//     //         }
//     //     }
//     // }
// }
// 
// pub fn verify_wallets(vault_id: u64, policy: &PolicyType) {
//     // match policy {
//     //     PolicyType::ThresholdPolicy(p) => {
//     //         match p.wallets.borrow() {
//     //             None => {}
//     //             Some(wallets) => {
//     //                 let vault = vault_service::get_by_id(&vault_id);
//     //                 for w in wallets {
//     //                     if !vault.wallets.contains(w) {
//     //                         trap("Stop it!!! Not your wallet!!!")
//     //                     }
//     //                 }
//     //             }
//     //         }
//     //     }
//     // }
// }
