use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::{get_member_by_id, restore_member};
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberUpdateNameTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateNameTransaction {
    pub common: BasicTransactionFields,
    pub id: String,
    pub name: String,
}

impl MemberUpdateNameTransaction {
    fn new(state: TransactionState, id: String, name: String) -> Self {
        MemberUpdateNameTransaction {
            common: BasicTransactionFields::new(state, TrType::MemberUpdateName, true),
            id,
            name,
        }
    }
}

#[async_trait]
impl ITransaction for MemberUpdateNameTransaction {
    async fn execute(&self, state: VaultState) -> VaultState {
        let mut m = get_member_by_id(&self.id, &state);
        m.name = self.name.clone();
        restore_member(m, state)
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateNameTransaction = self.clone();
        TransactionCandid::MemberUpdateNameTransactionV(trs)
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUpdateNameTransactionRequest {
    pub member: String,
    pub name: String,
}

pub struct MemberUpdateNameTransactionBuilder {
    pub id: String,
    pub name: String,
}

impl MemberUpdateNameTransactionBuilder {
    pub fn init(request: MemberUpdateNameTransactionRequest) -> Self {
        return MemberUpdateNameTransactionBuilder {
            id: request.member,
            name: request.name,
        };
    }
}

impl TransactionBuilder for MemberUpdateNameTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUpdateNameTransaction::new(
            state,
            self.id.clone(),
            self.name.clone(),
        );
        Box::new(trs)
    }
}



