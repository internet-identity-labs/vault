use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::ObjectState::Active;
use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
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
            common: BasicTransactionFields::new(state, TrType::MemberUnarchive, true),
            id
        }
    }
}


pub struct MemberUnarchiveTransactionBuilder {
    pub id: String,
}

impl TransactionBuilder for MemberUnarchiveTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUnarchiveTransaction::new(
            state,
            self.id.clone(),
        );
        Box::new(trs)
    }
}

impl MemberUnarchiveTransactionBuilder {
    pub fn init(id: String) -> Self {
        return MemberUnarchiveTransactionBuilder {
            id,
        };
    }
}

#[async_trait]
impl ITransaction for MemberUnarchiveTransaction {

    async fn execute(&self, state: VaultState) -> VaultState {
       let mut  m = get_member_by_id(&self.id, &state);
        m.state = Active;
        restore_member(m, state)
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUnarchiveTransaction = self.clone();
        TransactionCandid::MemberUnarchiveTransactionV(trs)
    }

}



