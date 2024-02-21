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
use crate::transaction::wallet::wallet::{Wallet, WalletType};

impl_basic_for_transaction!(WalletCreateTransactionV2);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletCreateTransactionV2 {
    common: BasicTransactionFields,
    uid: String,
    name: String,
    network: Network,
    wallet_type: WalletType,
}

impl WalletCreateTransactionV2 {
    fn new(state: TransactionState, batch_uid: Option<String>, uid: String, name: String, network: Network, wallet_type: WalletType) -> Self {
        WalletCreateTransactionV2 {
            common: BasicTransactionFields::new(state, batch_uid, true),
            uid,
            name,
            network,
            wallet_type,
        }
    }
}

#[async_trait]
impl ITransaction for WalletCreateTransactionV2 {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        if state.wallets.iter().find(|p| p.uid.eq(&self.uid)).is_some() {
            self.set_state(Rejected);
            self.common.error = Some(UIDAlreadyExists);
            return state;
        }

        let w = Wallet::new(self.uid.clone(),
                            self.name.clone(),
                            self.network.clone(),
                            self.wallet_type.clone(),
        );
        state.wallets.push(w);
        self.set_state(Executed);
        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletCreateTransactionV2 = self.clone();
        TransactionCandid::WalletCreateTransactionV2(trs)
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletCreateTransactionRequestV2 {
    pub network: Network,
    pub name: String,
    pub uid: String,
    pub batch_uid: Option<String>,
    pub wallet_type: WalletType,
}

pub struct WalletCreateTransactionBuilderV2 {
    request: WalletCreateTransactionRequestV2,
}

impl WalletCreateTransactionBuilderV2 {
    pub fn init(request: WalletCreateTransactionRequestV2) -> Self {
        return WalletCreateTransactionBuilderV2 {
            request,
        };
    }
}


impl TransactionBuilder for WalletCreateTransactionBuilderV2 {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = WalletCreateTransactionV2::new(state,
                                                 self.request.batch_uid.clone(),
                                                 self.request.uid.clone(),
                                                 self.request.name.clone(),
                                                 self.request.network.clone(),
                                                 self.request.wallet_type.clone(),
        );
        Box::new(trs)
    }
}


