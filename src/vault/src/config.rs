use std::cell::RefCell;

use candid::CandidType;
use candid::Deserialize;
use candid::Principal;
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Conf {
    pub ledger_canister_id: Principal,
    pub origins: Option<Vec<String>>,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
            origins: Default::default(),
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}
