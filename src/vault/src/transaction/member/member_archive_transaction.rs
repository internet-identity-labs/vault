use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use crate::enums::ObjectState::Archived;

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::vault_service::VaultRole;
use crate::transaction::basic_transaction::{ Basic};

impl_basic_for_transaction!(MemberArchiveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberArchiveTransaction {
    pub common: BasicTransactionFields,
    pub id: String
}

impl MemberArchiveTransaction {
    fn new(state: TransactionState, id:String) -> Self {
        MemberArchiveTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberArchive),
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
        let trs = MemberArchiveTransaction::new(
            state,
            self.id.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl TransactionNew for MemberArchiveTransaction {

    async fn execute(&self) {
       let mut  m = get_member_by_id(&self.id);
        m.state = Archived;
        restore_member(m); //TODO use ref
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberArchiveTransaction = self.clone();
        TransactionCandid::MemberArchiveTransaction(trs)
    }

}



