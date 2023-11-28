use crate::enums::TransactionState;
use crate::enums::TransactionState::{Blocked, Pending};
use crate::security_service::verify_caller;
use crate::transaction::transaction::ITransaction;
use crate::transaction::transactions_service::is_blocked;

pub trait TransactionBuilder {
    fn define_initial_state(&mut self) -> TransactionState {
        if is_blocked(|tr| {
            self.get_block_predicate(tr)
        }) { Blocked } else { Pending }
    }
    fn get_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        return get_vault_state_block_predicate(tr);
    }
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction>;
    fn build(&mut self) -> Box<dyn ITransaction> {
        let initial_state = self.define_initial_state();
        let trs = self.build_dyn_transaction(initial_state);
        verify_caller(trs.get_accepted_roles());
        trs
    }
}

pub fn get_vault_state_block_predicate(tr: &Box<dyn ITransaction>) -> bool {
    return tr.get_common_ref().is_vault_state;
}
