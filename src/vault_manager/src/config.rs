use std::cell::RefCell;

use candid::CandidType;
use candid::Deserialize;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Conf {
    pub origins: Vec<String>,
    pub initial_cycles_balance: u128,
    pub payment_cycles: u64,
    pub repo_canister_id: String,
    pub destination_address: String,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            payment_cycles: 100000000,
            origins: Default::default(),
            initial_cycles_balance: 500_000_000_000,
            repo_canister_id: "7jlkn-paaaa-aaaap-abvpa-cai".to_string(),
            destination_address: "4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e".to_string(),
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}
