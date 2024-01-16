use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError::WalletNotExists;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::restore_wallet;

impl_basic_for_transaction!(WalletUpdateNameTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletUpdateNameTransaction {
    common: BasicTransactionFields,
    uid: String,
    name: String,
}

impl WalletUpdateNameTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, uid: String, name: String) -> Self {
        WalletUpdateNameTransaction {
            common: BasicTransactionFields::new(state, batch_uid, true),
            uid,
            name,
        }
    }
}

#[async_trait]
impl ITransaction for WalletUpdateNameTransaction {
    async fn execute(&mut self, state: VaultState) -> VaultState {
        match state.wallets.iter()
            .find(|w| w.uid.eq(&self.uid)) {
            None => {
                self.set_state(Rejected);
                self.common.error = Some(WalletNotExists);
                state
            }
            Some(w) => {
                let mut wallet = w.clone();
                wallet.modified_date = time();
                wallet.name = self.name.clone();
                self.set_state(Executed);
                restore_wallet(wallet, state)
            }
        }
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletUpdateNameTransaction = self.clone();
        TransactionCandid::WalletUpdateNameTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletUpdateNameTransactionRequest {
    uid: String,
    name: String,
    batch_uid: Option<String>,
}

pub struct WalletUpdateNameTransactionBuilder {
    request: WalletUpdateNameTransactionRequest,
}

impl WalletUpdateNameTransactionBuilder {
    pub fn init(request: WalletUpdateNameTransactionRequest) -> Self {
        return WalletUpdateNameTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for WalletUpdateNameTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = WalletUpdateNameTransaction::new(state,
                                                   self.request.batch_uid.clone(),
                                                   self.request.uid.clone(),
                                                   self.request.name.clone());
        Box::new(trs)
    }
}

