use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::enums::TransactionState;
use crate::impl_basic_for_transaction;
use crate::policy_service::Currency;
use crate::transaction::basic_transaction::Basic;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::transaction::{TransactionCandid, TransactionNew, TrType};
use crate::transaction::transaction_builder::TransactionBuilder;
use crate::transaction::wallet::wallet::{store_wallet, Wallet};

impl_basic_for_transaction!(WalletCreateTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WalletCreateTransaction {
    pub common: BasicTransactionFields,
    pub uid: String,
    pub name: String,
    pub currency: Currency,
}

impl WalletCreateTransaction {
    fn new(state: TransactionState, uid: String, name: String) -> Self {
        WalletCreateTransaction {
            common: BasicTransactionFields::new(state, TrType::WalletCreate),
            uid,
            name,
            currency: Currency::ICP,
        }
    }
}

impl WalletCreateTransactionBuilder {
    pub fn init(uid: String, transaction_type: TrType, name: String, currency: Currency) -> Self {
        return WalletCreateTransactionBuilder {
            uid,
            name,
            currency,
            tr_type: transaction_type,
        };
    }
}

pub struct WalletCreateTransactionBuilder {
    uid: String,
    name: String,
    currency: Currency,
    tr_type: TrType,
}

impl TransactionBuilder for WalletCreateTransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew> {
        let trs = WalletCreateTransaction::new(state,
                                               self.uid.clone(),
                                               self.name.clone());
        Box::new(trs)
    }
}

#[async_trait]
impl TransactionNew for WalletCreateTransaction {

    async fn execute(&self) {
        store_wallet(Wallet::new(self.uid.clone(), self.name.clone()))
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: WalletCreateTransaction = self.clone();
        TransactionCandid::WalletCreateTransaction(trs)
    }
}



