use std::cell::RefCell;

use candid::CandidType;
use candid::Deserialize;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Conf {
    pub origins: Vec<String>,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            origins: Default::default(),
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}
