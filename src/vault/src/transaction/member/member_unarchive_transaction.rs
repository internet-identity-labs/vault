use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState::Active;
use crate::enums::TransactionState;
use crate::enums::TransactionState::Rejected;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberUnarchiveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUnarchiveTransaction {
    pub common: BasicTransactionFields,
    pub member_id: String
}

impl MemberUnarchiveTransaction {
    fn new(state: TransactionState, member_id:String) -> Self {
        MemberUnarchiveTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUnarchive, true),
            member_id: member_id
        }
    }
}


pub struct MemberUnarchiveTransactionBuilder {
    pub member_id: String,
}

impl TransactionBuilder for MemberUnarchiveTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUnarchiveTransaction::new(
            state,
            self.member_id.clone(),
        );
        Box::new(trs)
    }
}

impl MemberUnarchiveTransactionBuilder {
    pub fn init(member_id: String) -> Self {
        return MemberUnarchiveTransactionBuilder {
            member_id,
        };
    }
}

#[async_trait]
impl ITransaction for MemberUnarchiveTransaction {

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
                member.state = Active;
                self.set_state(TransactionState::Executed);
                restore_member(member, state)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUnarchiveTransaction = self.clone();
        TransactionCandid::MemberUnarchiveTransactionV(trs)
    }

}



