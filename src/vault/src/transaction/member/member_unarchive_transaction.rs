use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState::Active;
use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::Basic;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberUnarchiveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUnarchiveTransaction {
    pub common: BasicTransactionFields,
    pub id: String
}

impl MemberUnarchiveTransaction {
    fn new(state: TransactionState, id:String) -> Self {
        MemberUnarchiveTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUnarchive),
            id
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
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = MemberUnarchiveTransaction::new(
            state,
            self.id.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl TransactionNew for MemberUnarchiveTransaction {

    async fn execute(&self) {
       let mut  m = get_member_by_id(&self.id);
        m.state = Active;
        restore_member(m); //TODO use ref
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUnarchiveTransaction = self.clone();
        TransactionCandid::MemberArchiveTransaction(trs)
    }

}



