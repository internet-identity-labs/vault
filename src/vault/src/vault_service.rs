use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use candid::{CandidType, Deserialize};
use ic_cdk::trap;
use serde::{Serialize};

use crate::enums::ObjectState;
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
