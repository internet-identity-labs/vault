use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState::Archived;
use crate::enums::TransactionState;
use crate::{impl_basic_for_transaction};
use crate::enums::TransactionState::Rejected;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberArchiveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberArchiveTransaction {
    pub common: BasicTransactionFields,
    pub member_id: String,
}

impl MemberArchiveTransaction {
    fn new(state: TransactionState, id: String) -> Self {
        MemberArchiveTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberArchive, true),
            member_id: id,
        }
    }
}

impl MemberArchiveTransactionBuilder {
    pub fn init(id: String) -> Self {
        return MemberArchiveTransactionBuilder {
            id,
        };
    }
}

pub struct MemberArchiveTransactionBuilder {
    pub id: String,
}

impl TransactionBuilder for MemberArchiveTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberArchiveTransaction::new(
            state,
            self.id.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for MemberArchiveTransaction {
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
                member.state = Archived;
                let state =  restore_member(member, state);
                self.set_state(TransactionState::Executed);
                state
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberArchiveTransaction = self.clone();
        TransactionCandid::MemberArchiveTransactionV(trs)
    }

}



