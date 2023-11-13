use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::enums::{Network, TransactionState};
use crate::enums::TransactionState::{Blocked, Pending};
use crate::transaction::quorum::quorum::get_vault_admin_block_predicate;
use crate::transaction::transaction::TransactionNew;
use crate::transaction::transactions_service::is_blocked;

pub trait TransactionBuilder {
    fn define_initial_state(&mut self) -> TransactionState {
        if is_blocked(|tr| {
            self.get_block_predicate(tr)
        }) { Blocked } else { Pending }
    }
    fn get_block_predicate(&mut self, tr: &Box<dyn TransactionNew>) -> bool {
        return get_vault_admin_block_predicate(tr);
    }
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn TransactionNew>;
    fn build(&mut self) -> Box<dyn TransactionNew> {
        let initial_state = self.define_initial_state();
        self.build_dyn_transaction(initial_state)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct WalletCreateTransaction {
    pub network: Network,
    pub name: String,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TransferTransaction {
    pub wallet: String,
    pub to: String,
    pub amount: u64,
    pub block_index: Option<BlockIndex>,
}
