use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Failed};
use crate::errors::VaultError::{MemberAlreadyExists, MemberNotExists};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{Account, calculate_id, restore_member};
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberExtendICRC1AccountTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberExtendICRC1AccountTransaction {
    pub common: BasicTransactionFields,
    pub account: Account,
}

impl MemberExtendICRC1AccountTransaction {
    pub fn new(state: TransactionState, batch_uid: Option<String>, account: Account) -> Self {
        MemberExtendICRC1AccountTransaction {
            common: BasicTransactionFields::new(state, batch_uid, true),
            account,
        }
    }
}

#[async_trait]
impl ITransaction for MemberExtendICRC1AccountTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        let member_id = calculate_id(self.account.clone());
        match state.members.iter()
            .find(|x| x.member_id.eq_ignore_ascii_case(&member_id)) {
            None => {
                self.set_state(Failed);
                self.common.error = Some(MemberNotExists);
                state
            }
            Some(m) => {
                if m.account.is_some() {
                    self.set_state(Failed);
                    self.common.error = Some(MemberAlreadyExists);
                    return state;
                }
                let mut member = m.clone();
                member.account = Some(self.account.clone());
                self.set_state(Executed);
                restore_member(member, state)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberExtendICRC1AccountTransaction = self.clone();
        TransactionCandid::MemberExtendICRC1AccountTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberExtendICRC1AccountRequest {
    account: Account,
    batch_uid: Option<String>,
}

pub struct MemberExtendICRC1AccountBuilder {
    request: MemberExtendICRC1AccountRequest,
}

impl MemberExtendICRC1AccountBuilder {
    pub fn init(request: MemberExtendICRC1AccountRequest) -> Self {
        return MemberExtendICRC1AccountBuilder {
            request
        };
    }
}

impl TransactionBuilder for MemberExtendICRC1AccountBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberExtendICRC1AccountTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.account.clone(),
        );
        Box::new(trs)
    }
}


