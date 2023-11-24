use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::{Network, TransactionState};
use crate::enums::TransactionState::Executed;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{ITransaction, TransactionCandid, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::Wallet;

impl_basic_for_transaction!(WalletCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletCreateTransaction {
    pub common: BasicTransactionFields,
    pub uid: String,
    pub name: String,
    pub network: Network,
}

impl WalletCreateTransaction {
    fn new(state: TransactionState, uid: String, name: String, network: Network) -> Self {
        WalletCreateTransaction {
            common: BasicTransactionFields::new(state, TrType::WalletCreate, true),
            uid,
            name,
            network,
        }
    }
}

#[async_trait]
impl ITransaction for WalletCreateTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
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
}

pub struct WalletCreateTransactionBuilder {
    pub network: Network,
    pub name: String,
    pub uid: String,
}

impl WalletCreateTransactionBuilder {
    pub fn init(request: WalletCreateTransactionRequest, generated_random_address: String) -> Self {
        return WalletCreateTransactionBuilder {
            network: request.network,
            name: request.name,
            uid: generated_random_address,
        };
    }
}


impl TransactionBuilder for WalletCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = WalletCreateTransaction::new(state,
                                               self.uid.clone(),
                                               self.name.clone(),
                                               self.network.clone());
        Box::new(trs)
    }
}


