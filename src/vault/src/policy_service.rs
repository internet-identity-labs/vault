use std::collections::HashSet;

use candid::CandidType;
use ic_cdk::trap;
use serde::Deserialize;
use serde::Serialize;

use crate::enums::ObjectState;
use crate::memory::POLICIES;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Policy {
    pub id: u64,
    pub vault: u64,
    pub state: ObjectState,
    pub policy_type: PolicyType,
    pub created_date: u64,
    pub modified_date: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum PolicyType {
    #[serde(rename = "threshold_policy")]
    ThresholdPolicy(ThresholdPolicy)
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ThresholdPolicy {
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: Option<u8>,
    pub wallets: Option<Vec<String>>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum Currency {
    ICP,
}

pub fn register_policy(vault: u64, policy_type: PolicyType) -> Policy {
    POLICIES.with(|policies| {
        let ps = Policy {
            id: (policies.borrow().len() + 1) as u64,
            vault,
            state: ObjectState::Active,
            policy_type,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
        };
        policies.borrow_mut().insert(ps.id, ps.clone());
        ps
    })
}

pub fn restore_policy(mut policy: Policy) -> Policy {
    POLICIES.with(|policies| {
        policy.modified_date = ic_cdk::api::time();
        policies.borrow_mut().insert(policy.id, policy.clone());
        policy
    })
}


pub fn update_policy(ps: Policy) -> Policy {
    let mut old = get_by_id(ps.id);
    old.policy_type = ps.policy_type;
    old.state = ps.state;
    restore_policy(old.clone())
}


pub fn get_by_id(id: u64) -> Policy {
    POLICIES.with(|policies| {
        match policies.borrow().get(&id) {
            None => {
                trap("Not registered")
            }
            Some(policy) => {
                policy.clone()
            }
        }
    })
}

pub fn get(ids: HashSet<u64>) -> Vec<Policy> {
    POLICIES.with(|policies| {
        let mut result: Vec<Policy> = Default::default();
        for id in ids {
            match policies.borrow_mut().get(&id) {
                None => {
                    trap("Nonexistent key error")
                }
                Some(v) => { result.push(v.clone()) }
            }
        }
        result
    })
}

pub fn define_correct_policy(ids: HashSet<u64>, amount: u64, wallet: &String) -> Policy {
    let policy = get(ids).into_iter()
        //define policies related to requested wallet
        .map(|l| match l.policy_type.clone() {
            PolicyType::ThresholdPolicy(threshold_policy) => {
                match threshold_policy.wallets {
                    Some(x) => {
                        if x.contains(wallet)
                        { Some((l, threshold_policy.amount_threshold, threshold_policy.member_threshold, x)) } else { None }
                    }
                    None => {
                        Some((l, threshold_policy.amount_threshold, threshold_policy.member_threshold, vec![]))
                    }
                }
            }
        }
        )
        .filter(|l| l.is_some())
        .map(|l| l.unwrap())
        //find all policies with thresholdAmount less or equal to actual amount
        .filter(|l| l.1 <= amount)
        .reduce(|a, b|
            //find closest (biggest) greaterThan
            if a.1 > b.1 { a } else if b.1 > a.1 { b }
            //if 2 policies with same greaterThan
            //take assigned to wallet
            else if a.3.contains(wallet) && !b.3.contains(wallet) {
                a
            } else if b.3.contains(wallet) && !a.3.contains(wallet) {
                b
            }
            //if both assigned to a wallet
            else if a.3.contains(wallet) && b.3.contains(wallet) {
                //find more strict requirement for amount of members
                if a.2.is_none() { //if one of wallets contains all members - take it
                    a
                } else if b.2.is_none() {
                    b
                } else if a.2.unwrap() > b.2.unwrap() {
                    a
                } else { b }
            }
            //take first one with members All (that means Threshold Policies are equal or a more strict)
            else if a.2.is_none() {
                a
            } else if b.2.is_none() {
                b
            }
            //find more strict requirement for amount of members
            else if a.2.unwrap() > b.2.unwrap() {
                a
            } else if b.2.unwrap() > a.2.unwrap() {
                b
            } else {
                //should never trap
                trap("Define policy error")
            }
        )
        .map(|l| l.0);
    match policy {
        None => {
            trap("Unable to find the policy!")
        }
        Some(required) => {
            required
        }
    }
}