use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{TransactionCandid, ITransaction, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::restore_wallet;

impl_basic_for_transaction!(WalletUpdateNameTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletUpdateNameTransaction {
    pub common: BasicTransactionFields,
    pub uid: String,
    pub name: String,
}

impl WalletUpdateNameTransaction {
    fn new(state: TransactionState, uid: String, transaction_type: TrType, name: String) -> Self {
        WalletUpdateNameTransaction {
            common: BasicTransactionFields::new(state, transaction_type, true),
            uid,
            name,
        }
    }
}

#[async_trait]
impl ITransaction for WalletUpdateNameTransaction {
    async fn execute(&mut self, state: VaultState) -> VaultState {
        let mut wallet = state.wallets.iter()
            .find(|w| w.uid.eq(&self.uid))
            .unwrap().clone();
        wallet.modified_date = time();
        wallet.name = self.name.clone();
        restore_wallet(wallet, state)
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletUpdateNameTransaction = self.clone();
        TransactionCandid::WalletUpdateTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletUpdateNameTransactionRequest {
    pub uid: String,
    pub name: String,
}

pub struct WalletUpdateNameTransactionBuilder {
    request: WalletUpdateNameTransactionRequest
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
                                                   self.request.uid.clone(),
                                                   TrType::WalletUpdateName,
                                                   self.request.name.clone());
        Box::new(trs)
    }
}

