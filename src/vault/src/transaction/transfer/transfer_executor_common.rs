use std::convert::TryFrom;

use hex::FromHexError;
use ic_ledger_types::{AccountIdentifier, BlockIndex};

use crate::enums::TransactionState::{Executed, Failed};
use crate::errors::VaultError::CanisterReject;
use crate::state::VaultState;
use crate::transaction::transaction::ITransaction;
use crate::transfer_service::transfer;
use crate::util::to_array;

pub trait TransferExecutor: ITransaction {
    fn get_address(&self) -> String;
    fn get_amount(&self) -> u64;
    fn get_wallet(&self) -> String;
    fn set_block_index(&mut self, bi: Option<BlockIndex>);
    async fn execute_transfer(&mut self, state: VaultState) -> VaultState {
        let to_decoded = match hex::decode(self.get_address().clone()) {
            Ok(x) => { x }
            Err(err) => {
                match err {
                    FromHexError::InvalidHexCharacter { c, index } => {
                        self.set_state(Failed);
                        self.get_common_mut().error = Some(CanisterReject { message: format!("Invalid hex character {} at index {}", c, index) });
                    }
                    FromHexError::OddLength => {
                        self.set_state(Failed);
                        self.get_common_mut().error = Some(CanisterReject { message: format!("OddLength") });
                    }
                    FromHexError::InvalidStringLength => {
                        self.set_state(Failed);
                        self.get_common_mut().error = Some(CanisterReject { message: format!("InvalidStringLength") });
                    }
                }
                return state;
            }
        };
        let to = match AccountIdentifier::try_from(to_array(to_decoded)) {
            Ok(x) => { x }
            Err(err) => {
                self.set_state(Failed);
                self.get_common_mut().error = Some(CanisterReject { message: err.to_string() });
                return state;
            }
        };
        let transfer = transfer(self.get_amount(), to, self.get_wallet().clone(), None)
            .await;
        match transfer {
            Ok(result) => {
                self.set_block_index(Some(result));
                self.set_state(Executed);
            }
            Err(message) => {
                self.set_state(Failed);
                self.get_common_mut().error = Some(CanisterReject { message });
            }
        }
        state
    }
}


#[macro_export]
macro_rules! impl_transfer_executor_for_transaction {
    ($type:ty) => {
        impl TransferExecutor for $type {
            fn get_address(&self) -> String {
              self.address.clone()
              }

            fn set_block_index(&mut self, bi: Option<BlockIndex>) {
              self.block_index=bi
              }

           fn get_wallet(&self) -> String {
              self.wallet.clone()
              }

           fn get_amount(&self) -> u64 {
               self.amount.clone()
              }
        }
    };
}
