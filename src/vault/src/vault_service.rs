use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_cdk::trap;
use serde::{Serialize};

use crate::enums::ObjectState;
use crate::memory::VAULTS;
use crate::User;
use crate::VaultRole::Admin;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Vault {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub wallets: HashSet<String>,
    pub policies: HashSet<u64>,
    pub members: HashSet<VaultMember>,
    pub state: ObjectState,
    pub created_date: u64,
    pub modified_date: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Eq, Serialize)]
pub struct VaultMember {
    pub user_uuid: String,
    pub role: VaultRole,
    pub name: Option<String>,
    pub state: ObjectState,
}

impl Hash for VaultMember {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_uuid.hash(state)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Copy, Eq, PartialEq, Serialize)]
pub enum VaultRole {
    Admin,
    Member,
}

impl PartialEq for Vault {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialEq for VaultMember {
    fn eq(&self, other: &Self) -> bool {
        self.user_uuid.eq(&other.user_uuid)
    }
}

pub fn register(user_uuid: String, name: String, description: Option<String>) -> Vault {
    VAULTS.with(|vaults| {
        let vault_id = (vaults.borrow().len() + 1) as u64;
        let mut participants: HashSet<VaultMember> = Default::default();
        let owner = VaultMember { user_uuid, role: Admin, name: None, state: ObjectState::Active };
        participants.insert(owner);
        let vault_new: Vault = Vault {
            id: vault_id,
            name,
            description,
            wallets: hashset![],
            policies: hashset![],
            members: participants,
            state: ObjectState::Active,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
        };
        vaults.borrow_mut().insert(vault_id, vault_new.clone());
        return vault_new;
    })
}

pub fn get(ids: HashSet<u64>) -> Vec<Vault> {
    VAULTS.with(|vaults| {
        let mut result: Vec<Vault> = Default::default();
        let borrowed = vaults.borrow();
        for id in ids {
            match borrowed.get(&id) {
                None => {
                    trap("Nonexistent key error")
                }
                Some(v) => { result.push(v.clone()) }
            }
        }
        result
    })
}

pub fn get_by_id(id: &u64) -> Vault {
    VAULTS.with(|vaults| {
        match vaults.borrow().get(id) {
            None => {
                trap("Nonexistent key error")
            }
            Some(v) => {
                v.clone()
            }
        }
    })
}

pub fn add_vault_member(vault_id: u64, user: &User, role: VaultRole, name: Option<String>, state: ObjectState) -> Vault {
    let mut vault = get_by_id(&vault_id);
    let vm = VaultMember {
        user_uuid: user.address.clone(),
        role,
        name,
        state,
    };
    vault.members.replace(vm);
    restore(&vault)
}

pub fn restore(vault: &Vault) -> Vault {
    VAULTS.with(|vaults| {
        let mut v = vault.clone();
        v.modified_date = ic_cdk::api::time();
        match vaults.borrow_mut().insert(v.id, v.clone()) {
            None => {
                trap("No such vault")
            }
            Some(_) => {
                v
            }
        }
    })
}


pub fn update(vault: Vault) -> Vault {
    let mut old = get_by_id(&vault.id);
    old.state = vault.state;
    old.description = vault.description;
    old.name = vault.name;
    restore(&old)
}
