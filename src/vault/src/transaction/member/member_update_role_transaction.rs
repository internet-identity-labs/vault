use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::vault_service::VaultRole;
use crate::transaction::basic_transaction::{BasicTransaction};

impl_basic_for_transaction!(MemberUpdateRoleTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateRoleTransaction {
    pub common: BasicTransactionFields,
    pub member_id: String,
    pub role: VaultRole,
}

impl MemberUpdateRoleTransaction {
    fn new(state: TransactionState, member_id: String, role: VaultRole) -> Self {
        MemberUpdateRoleTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUpdateRole, true),
            member_id,
            role,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUpdateRoleTransactionRequest {
    pub member_id: String,
    pub role: VaultRole,
}

pub struct MemberUpdateRoleTransactionBuilder {
    pub member_id: String,
    pub role: VaultRole,
}

impl MemberUpdateRoleTransactionBuilder {
    pub fn init(request: MemberUpdateRoleTransactionRequest) -> Self {
        return MemberUpdateRoleTransactionBuilder {
            member_id: request.member_id,
            role: request.role,
        };
    }
}

impl TransactionBuilder for MemberUpdateRoleTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUpdateRoleTransaction::new(
            state,
            self.member_id.clone(),
            self.role.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for MemberUpdateRoleTransaction {

    async fn execute(&mut self, state: VaultState) -> VaultState {
        match state.members.iter()
            .find(|x| x.member_id.eq(&self.member_id)) {
            None => {
                self.set_state(Rejected);
                self.common.memo = Some("No such member".to_string());
                state
            }
            Some(m) => {
                let mut member = m.clone();
                member.role = self.role.clone();
                self.set_state(Executed);
                restore_member(member, state)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateRoleTransaction = self.clone();
        TransactionCandid::MemberUpdateRoleTransactionV(trs)
    }

}



