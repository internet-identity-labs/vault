use ic_cdk::trap;

use crate::transaction::member::members::get_caller_role;
use crate::vault_service::VaultRole;

pub fn verify_caller(accepted_roles: Vec<VaultRole>) {
    let caller_role = get_caller_role();
    if !accepted_roles.contains(&caller_role) {
        trap("Not permitted")
    }
}
