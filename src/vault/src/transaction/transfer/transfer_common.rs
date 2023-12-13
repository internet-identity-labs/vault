use crate::state::get_current_state;
use crate::transaction::transaction::{get_vault_state_block_predicate, ITransaction, TransactionCandid};

/*
if you make any changes to this file
 you need to unskip and run
 top_up_transaction.test.ts
*/

pub trait TransferHelper: ITransaction {
    fn get_wallet(&self) -> String;
    fn get_amount(&self) -> u64;
    fn set_policy(&mut self, x: Option<String>);

    fn define_transfer_threshold(&mut self) -> Result<u8, String> {
        let state = get_current_state();
        let wallet = self.get_wallet();
        let amount = self.get_amount();
        let policy = state.policies.iter()
            .filter(|p| p.wallets.contains(&wallet))
            .filter(|p| p.amount_threshold < amount)
            .max_by(|a, b| {
                a.amount_threshold.cmp(&b.amount_threshold)
            });
        match policy {
            None => {
                Err("No suitable policy".to_string())
            }
            Some(x) => {
                self.set_policy(Some(x.uid.clone()));
                self.set_threshold(x.member_threshold);
                Ok(x.member_threshold)
            }
        }
    }

    fn get_transfer_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        if tr.get_id() >= self.get_id() {
            return false;
        }
        if get_vault_state_block_predicate(tr) {
            return true;
        }
        if let TransactionCandid::TransferTransactionV(transfer) = tr.to_candid() {
            return transfer.get_wallet() == self.get_wallet();
        }
        if let TransactionCandid::TopUpTransactionV(transfer) = tr.to_candid() {
            return transfer.get_wallet() == self.get_wallet();
        }
        false
    }
}


#[macro_export]
macro_rules! impl_transfer_for_transaction {
    ($type:ty) => {
        impl TransferHelper for $type {
           fn get_wallet(&self) -> String {
              self.wallet.clone()
              }

           fn get_amount(&self) -> u64 {
               self.amount.clone()
              }

           fn set_policy(&mut self, x: Option<String>) {
               self.policy = x;
              }
        }
    };
}
