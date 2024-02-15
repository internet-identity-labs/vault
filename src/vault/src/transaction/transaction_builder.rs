use crate::enums::TransactionState;
use crate::enums::TransactionState::Blocked;
use crate::transaction::transaction::ITransaction;

pub trait TransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction>;
    async fn build(&mut self) -> Box<dyn ITransaction> {
        let trs = self.build_dyn_transaction(Blocked).await;
        trs
    }
}
