use std::hash::{Hash, Hasher};

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::enums::ObjectState;

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


impl ToString for VaultRole {
    fn to_string(&self) -> String {
        match self {
            VaultRole::Admin => String::from("Admin"),
            VaultRole::Member => String::from("Member"),
        }
    }
}


impl PartialEq for VaultMember {
    fn eq(&self, other: &Self) -> bool {
        self.user_uuid.eq(&other.user_uuid)
    }
}
