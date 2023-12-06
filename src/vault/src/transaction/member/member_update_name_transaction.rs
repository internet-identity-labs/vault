use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::member::members::restore_member;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberUpdateNameTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberUpdateNameTransaction {
    common: BasicTransactionFields,
    member_id: String,
    name: String,
}

impl MemberUpdateNameTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, member: String, name: String) -> Self {
        MemberUpdateNameTransaction {
            common: BasicTransactionFields::new(state, batch_uid, TrType::MemberUpdateName, true),
            member_id: member,
            name,
        }
    }
}

#[async_trait]
impl ITransaction for MemberUpdateNameTransaction {
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
                member.name = self.name.clone();
                self.set_state(Executed);
                restore_member(member, state)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberUpdateNameTransaction = self.clone();
        TransactionCandid::MemberUpdateNameTransactionV(trs)
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberUpdateNameTransactionRequest {
    member_id: String,
    name: String,
    batch_uid: Option<String>,
}

pub struct MemberUpdateNameTransactionBuilder {
    request: MemberUpdateNameTransactionRequest,
}

impl MemberUpdateNameTransactionBuilder {
    pub fn init(request: MemberUpdateNameTransactionRequest) -> Self {
        return MemberUpdateNameTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for MemberUpdateNameTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberUpdateNameTransaction::new(
            state,
            self.request.batch_uid.clone(),
            self.request.member_id.clone(),
            self.request.name.clone(),
        );
        Box::new(trs)
    }
}



