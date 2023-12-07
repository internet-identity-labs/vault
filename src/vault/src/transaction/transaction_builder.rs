use crate::enums::TransactionState;
use crate::enums::TransactionState::Blocked;
use crate::security_service::verify_caller;
use crate::transaction::transaction::ITransaction;

//TODO reorganize this trait
pub trait TransactionBuilder {
    fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction>;
    fn build(&mut self) -> Box<dyn ITransaction> {
        let trs = self.build_dyn_transaction(Blocked);
        trs
    }
}
