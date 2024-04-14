use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Failed};
use crate::enums::VaultRole::Admin;
use crate::errors::VaultError::{MemberNotExists, QuorumNotReachable};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(MemberRemoveTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct MemberRemoveTransaction {
    common: BasicTransactionFields,
    member_id: String,
}

impl MemberRemoveTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, id: String) -> Self {
        MemberRemoveTransaction {
            common: BasicTransactionFields::new(state, batch_uid,  true),
            member_id: id,
        }
    }
}

impl MemberRemoveTransactionBuilder {
    pub fn init(request: MemberRemoveTransactionRequest) -> Self {
        return MemberRemoveTransactionBuilder {
            request
        };
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct MemberRemoveTransactionRequest {
    member_id: String,
    batch_uid: Option<String>,
}

pub struct MemberRemoveTransactionBuilder {
    request: MemberRemoveTransactionRequest,
}

impl TransactionBuilder for MemberRemoveTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = MemberRemoveTransaction::new(
            state, self.request.batch_uid.clone(),
            self.request.member_id.clone(),
        );
        Box::new(trs)
    }
}

#[async_trait]
impl ITransaction for MemberRemoveTransaction {
    async fn execute(&mut self, state: VaultState) -> VaultState {
        match state.members.iter()
            .find(|mbr| mbr.member_id.eq_ignore_ascii_case(&self.member_id)) {
            None => {
                self.set_state(Failed);
                self.common.error = Some(MemberNotExists);
                state
            }
            Some(_) => {
                let mut state_sandbox = state.clone();
                state_sandbox.members.retain(|existing| existing.member_id != self.member_id);
                if state_sandbox.members.iter()
                    .filter(|m| m.role.eq(&Admin))
                    .count() < state.quorum.quorum as usize {
                    self.set_state(Failed);
                    self.common.error = Some(QuorumNotReachable);
                    state
                } else {
                    self.set_state(Executed);
                    state_sandbox
                }
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: MemberRemoveTransaction = self.clone();
        TransactionCandid::MemberRemoveTransactionV(trs)
    }
}



