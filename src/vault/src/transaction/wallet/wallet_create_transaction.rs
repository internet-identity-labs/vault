use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{Network, TransactionState};
use crate::enums::TransactionState::{Executed, Rejected};
use crate::errors::VaultError::UIDAlreadyExists;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::Wallet;

impl_basic_for_transaction!(WalletCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletCreateTransaction {
    common: BasicTransactionFields,
    uid: String,
    name: String,
    network: Network,
}

impl WalletCreateTransaction {
    fn new(state: TransactionState, batch_uid: Option<String>, uid: String, name: String, network: Network) -> Self {
        WalletCreateTransaction {
            common: BasicTransactionFields::new(state, batch_uid,  true),
            uid,
            name,
            network,
        }
    }
}

#[async_trait]
impl ITransaction for WalletCreateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        if state.wallets.iter().find(|p| p.uid.eq(&self.uid)).is_some() {
            self.set_state(Rejected);
            self.common.error = Some(UIDAlreadyExists);
            return state;
        }

        let w = Wallet::new(self.uid.clone(),
                            self.name.clone(),
                            self.network.clone());
        state.wallets.push(w);
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletCreateTransaction = self.clone();
        TransactionCandid::WalletCreateTransactionV(trs)
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletCreateTransactionRequest {
    pub network: Network,
    pub name: String,
    pub uid: String,
    pub batch_uid: Option<String>,
}

pub struct WalletCreateTransactionBuilder {
    request: WalletCreateTransactionRequest,
}

impl WalletCreateTransactionBuilder {
    pub fn init(request: WalletCreateTransactionRequest) -> Self {
        return WalletCreateTransactionBuilder {
            request,
        };
    }
}


impl TransactionBuilder for WalletCreateTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = WalletCreateTransaction::new(state,
                                               self.request.batch_uid.clone(),
                                               self.request.uid.clone(),
                                               self.request.name.clone(),
                                               self.request.network.clone());
        Box::new(trs)
    }
}


