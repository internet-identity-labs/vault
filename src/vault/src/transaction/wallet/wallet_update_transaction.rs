use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::{restore_wallet, Wallet};
use crate::transaction::basic_transaction::{ Basic};


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
            common: BasicTransactionFields::new(state, transaction_type),
            uid,
            name,
        }
    }
}

impl WalletUpdateNameTransactionBuilder {
    pub fn init(uid: String, name: String) -> Self {
        return WalletUpdateNameTransactionBuilder {
            uid,
            name,
        };
    }
}

pub struct WalletUpdateNameTransactionBuilder {
    uid: String,
    name: String,
}

impl TransactionBuilder for WalletUpdateNameTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = WalletUpdateNameTransaction::new(state,
                                                   self.uid.clone(),
                                                   TrType::WalletUpdateName,
                                                   self.name.clone());
        Box::new(trs)
    }
}

#[async_trait]
impl TransactionNew for WalletUpdateNameTransaction {

    async fn execute(&self) {
        restore_wallet(Wallet::new(self.uid.clone(), self.name.clone()))
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletUpdateNameTransaction = self.clone();
        TransactionCandid::WalletUpdateTransaction(trs)
    }

}

