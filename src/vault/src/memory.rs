use candid::Principal;
use ic_cdk::storage;
use crate::{Backup, Policy, Transaction, User, Vault, Wallet};
use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;
use std::cell::RefCell;
use std::collections::{HashMap};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VaultMemoryObject {
    pub vaults: Vec<Vault>,
    pub users: Vec<User>,
    pub wallets: Vec<Wallet>,
    pub transactions: Vec<Transaction>,
    pub policies: Vec<Policy>,
    pub conf: Option<Conf>
}


#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub ledger_canister_id: Principal,
    pub controllers: Option<Vec<Principal>>,
    pub origins: Option<Vec<String>>
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
            controllers: Default::default(),
            origins: Default::default()
        }
    }
}

thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
    pub static USERS: RefCell<HashMap<String, User>> = RefCell::new(Default::default());
    pub static VAULTS: RefCell<HashMap<u64, Vault>> = RefCell::new(Default::default());
    pub static WALLETS: RefCell<HashMap<String, Wallet>> = RefCell::new(Default::default());
    pub static POLICIES: RefCell<HashMap<u64, Policy>> = RefCell::new(Default::default());
    pub static TRANSACTIONS: RefCell<HashMap<u64, Transaction>> = RefCell::new(Default::default());
}

pub fn pre_upgrade() {
    let vaults: Vec<Vault> = VAULTS.with(|vaults| {
        vaults.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let wallets: Vec<Wallet> = WALLETS.with(|wallets| {
        wallets.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let users: Vec<User> = USERS.with(|users| {
        users.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let transactions: Vec<Transaction> = TRANSACTIONS.with(|transactions| {
        transactions.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let policies: Vec<Policy> = POLICIES.with(|policies| {
        policies.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let conf: Conf = CONF.with(|conf|{
        conf.borrow().clone()
    });
    let memory = VaultMemoryObject {
        vaults,
        users,
        wallets,
        policies,
        transactions,
        conf: Some(conf)
    };
    storage::stable_save((memory, )).unwrap();
}

pub fn get_all_json(from: u32, mut to: u32, obj: Backup) -> String {
    match obj {
        Backup::Vaults => {
            let mut raw: Vec<Vault> = VAULTS.with(|vaults| {
                vaults.borrow()
                    .iter()
                    .map(|l| l.1.clone())
                    .collect()
            });
            raw.sort_by(|a, b| a.modified_date.cmp(&b.modified_date));
            let len = raw.len() as u32;
            if to > len {
                to = len;
            }
            let resp = &raw[from as usize..to as usize];
            return serde_json::to_string(&resp).unwrap();
        }
        Backup::Wallets => {
            let mut raw: Vec<Wallet> = WALLETS.with(|wallets| {
                wallets.borrow()
                    .iter()
                    .map(|l| l.1.clone())
                    .collect()
            });
            raw.sort_by(|a, b| a.modified_date.cmp(&b.modified_date));
            let len = raw.len() as u32;
            if to > len {
                to = len;
            }
            let resp = &raw[from as usize..to as usize];
            return serde_json::to_string(&resp).unwrap();
        }
        Backup::Users => {
            let mut raw: Vec<User> = USERS.with(|users| {
                users.borrow()
                    .iter()
                    .map(|l| l.1.clone())
                    .collect()
            });
            raw.sort_by(|a, b| a.address.cmp(&b.address));
            let len = raw.len() as u32;
            if to > len {
                to = len;
            }
            let resp = &raw[from as usize..to as usize];
            return serde_json::to_string(&resp).unwrap();
        }
        Backup::Policies => {
            let mut raw: Vec<Policy> = POLICIES.with(|ps| {
                ps.borrow()
                    .iter()
                    .map(|l| l.1.clone())
                    .collect()
            });
            raw.sort_by(|a, b| a.created_date.cmp(&b.created_date));
            let len = raw.len() as u32;
            if to > len {
                to = len;
            }
            let resp = &raw[from as usize..to as usize];
            return serde_json::to_string(&resp).unwrap();
        }
        Backup::Transactions => {
            let mut raw: Vec<Transaction> = TRANSACTIONS.with(|ts| {
                ts.borrow()
                    .iter()
                    .map(|l| l.1.clone())
                    .collect()
            });
            raw.sort_by(|a, b| a.created_date.cmp(&b.created_date));
            let len = raw.len() as u32;
            if to > len {
                to = len;
            }
            let resp = &raw[from as usize..to as usize];
            return serde_json::to_string(&resp).unwrap();
        }
    }
}

pub fn count(obj: Backup) -> usize {
    match obj {
        Backup::Vaults => {
            VAULTS.with(|vaults| {
                vaults.borrow().keys().count()
            })
        }
        Backup::Wallets => {
           WALLETS.with(|wallets| {
                wallets.borrow().keys().count()
            })
        }
        Backup::Users => {
            USERS.with(|users| {
                users.borrow().keys().count()
            })
        }
        Backup::Policies => {
           POLICIES.with(|ps| {
                ps.borrow().keys().count()
            })
        }
        Backup::Transactions => {
            TRANSACTIONS.with(|ts| {
                ts.borrow().keys().count()
            })
        }
    }
}


pub fn post_upgrade() {
    let (mo, ): (VaultMemoryObject, ) = storage::stable_restore().unwrap();
    let mut vaults: HashMap<u64, Vault> = Default::default();
    for vault in mo.vaults {
        vaults.insert(vault.id, vault);
    }
    let mut wallets: HashMap<String, Wallet> = Default::default();
    for wallet in mo.wallets {
        wallets.insert(wallet.uid.clone(), wallet);
    }
    let mut users: HashMap<String, User> = Default::default();
    for user in mo.users {
        users.insert(user.address.clone(), user);
    }
    let mut policies: HashMap<u64, Policy> = Default::default();
    for policy in mo.policies {
        policies.insert(policy.id, policy);
    }
    let mut transactions: HashMap<u64, Transaction> = Default::default();
    for transaction in mo.transactions {
        transactions.insert(transaction.id, transaction);
    }
    let conf = mo.conf.unwrap_or(Conf::default());
    VAULTS.with(|storage| *storage.borrow_mut() = vaults);
    USERS.with(|storage| *storage.borrow_mut() = users);
    WALLETS.with(|storage| *storage.borrow_mut() = wallets);
    POLICIES.with(|storage| *storage.borrow_mut() = policies);
    TRANSACTIONS.with(|storage| *storage.borrow_mut() = transactions);
    CONF.with(|storage| *storage.borrow_mut() = conf);
}
