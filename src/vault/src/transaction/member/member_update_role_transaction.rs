use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::vault_service::VaultRole;
use crate::transaction::basic_transaction::{BasicTransaction};

impl_basic_for_transaction!(MemberUpdateRoleTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateRoleTransaction {
    pub common: BasicTransactionFields,
    pub id: String,
    pub role: VaultRole,
}

impl MemberUpdateRoleTransaction {
    fn new(state: TransactionState, id: String, role: VaultRole) -> Self {
        MemberUpdateRoleTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUpdateRole, true),
            id,
            role,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUpdateRoleTransactionRequest {
    pub member: String,
    pub role: VaultRole,
}

pub struct MemberUpdateRoleTransactionBuilder {
    pub id: String,
    pub role: VaultRole,
}

impl MemberUpdateRoleTransactionBuilder {
    pub fn init(request: MemberUpdateRoleTransactionRequest) -> Self {
        return MemberUpdateRoleTransactionBuilder {
            id: request.member,
            role: request.role,
        };
    }
}

impl TransactionBuilder for MemberUpdateRoleTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUpdateRoleTransaction::new(
            state,
            self.id.clone(),
            self.role.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for MemberUpdateRoleTransaction {

    async fn execute(&self, state: VaultState) -> VaultState {
       let mut  m = get_member_by_id(&self.id, &state);
        m.role = self.role.clone();
        restore_member(m, state)
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateRoleTransaction = self.clone();
        TransactionCandid::MemberUpdateRoleTransactionV(trs)
    }

}



