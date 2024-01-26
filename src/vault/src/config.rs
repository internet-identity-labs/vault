use std::cell::RefCell;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Conf {
    pub origins: Vec<String>,
    pub management_canister: String,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            origins: Default::default(),
            management_canister: "sgk26-7yaaa-aaaan-qaovq-cai".to_string(),
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}

pub fn get_management_canister() -> Principal {
    CONF.with(|c| Principal::from_text(c.borrow().management_canister.clone()).unwrap())
}