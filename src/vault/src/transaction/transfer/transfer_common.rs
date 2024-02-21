use crate::errors::VaultError;
use crate::errors::VaultError::{CouldNotDefinePolicy, WalletNotExists};
use crate::state::{get_current_state, VaultState};
use crate::transaction::policy::policy::Policy;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::wallet::wallet::{Wallet, WalletType};

/*
if you make any changes to this file
 you need to unskip and run
 top_up_transaction.test.ts
*/

pub trait TransferHelper: ITransaction {
    fn get_wallet(&self) -> String;
    fn get_amount(&self) -> u64;
    fn set_policy(&mut self, x: Option<String>);

    fn define_transfer_threshold(&mut self) -> Result<u8, VaultError> {
        let state = get_current_state();
        let wallet_uid = self.get_wallet();
        let amount = self.get_amount();

        let wallet = state.wallets.iter()
            .find(|w| w.uid.eq(&wallet_uid));

        if wallet.is_none() {
            return Err(WalletNotExists);
        }

        match wallet.unwrap().wallet_type {
            WalletType::Quorum => {
                Ok(state.quorum.quorum)
            }
            WalletType::Policy => {
                let policy = state.policies.iter()
                    .filter(|p| p.wallets.contains(&wallet_uid))
                    .filter(|p| p.amount_threshold < amount)
                    .max_by(|a, b| {
                        a.amount_threshold.cmp(&b.amount_threshold)
                    });
                match policy {
                    None => {
                        Err(CouldNotDefinePolicy)
                    }
                    Some(x) => {
                        self.set_policy(Some(x.uid.clone()));
                        self.set_threshold(x.member_threshold);
                        Ok(x.member_threshold)
                    }
                }
            }
        }
    }

    fn get_transfer_block_predicate(&mut self, tr: &Box<dyn ITransaction>) -> bool {
        if tr.get_id() >= self.get_id() {
            return false;
        }
        if tr.get_common_ref().is_vault_state {
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
