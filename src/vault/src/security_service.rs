use ic_cdk::trap;

use crate::enums::VaultRole;
use crate::state::STATE;
use crate::transaction::member::members::get_caller_role;
use crate::util::caller_to_address;

pub fn verify_caller(accepted_roles: Vec<VaultRole>) {
    let caller_role = get_caller_role();
    if !accepted_roles.contains(&caller_role) {
        trap("Not permitted")
    }
}

pub fn is_caller_registered() -> Result<(), String> {
    let caller = caller_to_address();
    match STATE.with(|mrs| {
        mrs.borrow().members.iter()
            .find(|m| m.member_id.eq_ignore_ascii_case(&caller))
            .map(|_| ())
    }) {
        None => {
            Err("Not registered".to_owned())
        }
        Some(_) => { Ok(()) }
    }
}
