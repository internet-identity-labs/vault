use std::cell::RefCell;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Conf {
    pub origins: Vec<String>,
    pub repo_canister: String,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            origins: Default::default(),
            repo_canister: "7jlkn-paaaa-aaaap-abvpa-cai".to_string(),
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}

pub fn get_repo_canister_id() -> Principal {
    CONF.with(|c| Principal::from_text(c.borrow().repo_canister.clone()).unwrap())
}