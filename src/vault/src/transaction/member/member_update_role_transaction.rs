use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{TransactionState, VaultRole};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::enums::VaultRole::Admin;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::restore_member;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberUpdateRoleTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateRoleTransaction {
    common: BasicTransactionFields,
    member_id: String,
    role: VaultRole,
}

impl MemberUpdateRoleTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, member_id: String, role: VaultRole) -> Self {
        MemberUpdateRoleTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::MemberUpdateRole, true),
            member_id,
            role,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUpdateRoleTransactionRequest {
    member_id: String,
    role: VaultRole,
    batch_uid: Option<String>,
}

pub struct MemberUpdateRoleTransactionBuilder {
    request: MemberUpdateRoleTransactionRequest,
}

impl MemberUpdateRoleTransactionBuilder {
    pub fn init(request: MemberUpdateRoleTransactionRequest) -> Self {
        return MemberUpdateRoleTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for MemberUpdateRoleTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUpdateRoleTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.member_id.clone(),
            self.request.role.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for MemberUpdateRoleTransaction {
    async fn execute(&mut self, state: VaultState) -> VaultState {
        match state.members.iter()
            .find(|mbr| mbr.member_id.eq(&self.member_id)) {
            None => {
                self.set_state(Rejected);
                self.common.memo = Some("No such member".to_string());
                state
            }
            Some(m) => {
                let mut state_sandbox = state.clone();
                let mut member = m.clone();
                member.role = self.role.clone();
                state_sandbox = restore_member(member, state_sandbox);
                if state_sandbox.members.iter()
                    .filter(|m| m.role.eq(&Admin))
                    .count() < state.quorum.quorum as usize {
                    self.set_state(Rejected);
                    state
                } else {
                    self.set_state(Executed);
                    state_sandbox
                }
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateRoleTransaction = self.clone();
        TransactionCandid::MemberUpdateRoleTransactionV(trs)
    }
}



