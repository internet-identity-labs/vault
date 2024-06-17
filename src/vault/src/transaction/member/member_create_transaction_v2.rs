use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{TransactionState, VaultRole};
use crate::errors::VaultError::MemberAlreadyExists;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{Account, calculate_id, Member};
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberCreateTransactionV2);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberCreateTransactionV2 {
    pub common: BasicTransactionFields,
    pub role: VaultRole,
    pub name: String,
    pub account: Account,
}

impl MemberCreateTransactionV2 {
    pub fn new(state: TransactionState, batch_uid: Option<String>, account: Account, name: String, role: VaultRole) -> Self {
        MemberCreateTransactionV2 {
            common: BasicTransactionFields::new(state, batch_uid, true),
            account,
            name,
            role,
        }
    }
}

#[async_trait]
impl ITransaction for MemberCreateTransactionV2 {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        let member = Member::new(self.account.clone(), self.role, self.name.clone());
        let member_id = calculate_id(self.account.clone());
        if state.members.iter()
            .any(|m| m.member_id.eq_ignore_ascii_case(&member_id)) {
            self.set_state(TransactionState::Failed);
            self.common.error = Some(MemberAlreadyExists);
            state
        } else {
            state.members.push(member);
            self.set_state(TransactionState::Executed);
            state
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberCreateTransactionV2 = self.clone();
        TransactionCandid::MemberCreateTransactionV2(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberCreateTransactionRequestV2 {
    account: Account,
    name: String,
    role: VaultRole,
    batch_uid: Option<String>,
}

pub struct MemberCreateTransactionBuilderV2 {
    request: MemberCreateTransactionRequestV2,
}

impl MemberCreateTransactionBuilderV2 {
    pub fn init(request: MemberCreateTransactionRequestV2) -> Self {
        return MemberCreateTransactionBuilderV2 {
            request
        };
    }
}

impl TransactionBuilder for MemberCreateTransactionBuilderV2 {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberCreateTransactionV2::new(
            state,
            self.request.batch_uid.clone(),
            self.request.account.clone(),
            self.request.name.clone(),
            self.request.role,
        );
        Box::new(trs)
    }
}


