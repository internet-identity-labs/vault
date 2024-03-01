use std::convert::TryInto;
use std::time::SystemTime;

use async_trait::async_trait;
use candid::{CandidType, Encode, Principal};
use ic_cdk::{id, print};
use ic_cdk::api::management_canister;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId};
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;
use ic_nervous_system_common::ledger::compute_neuron_staking_subaccount_bytes;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
// use ic_nervous_system_common::ledger;
// use icrc_ledger_types::icrc1::account::Account;
// use icrc_ledger_types::icrc1::transfer::NumTokens;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::enums::{TransactionState, VaultRole};
use crate::impl_basic_for_transaction;
use crate::state::VaultState;
use crate::transaction::basic_transaction::BasicTransaction;
use crate::transaction::basic_transaction::BasicTransactionFields;
use crate::transaction::nns::ecdsa::{get_principal, get_public_key, make_canister_call_via_ecdsa, prepare_canister_call_via_ecdsa};
use nns_governance_canister::types::{ListNeurons, manage_neuron_response, ManageNeuron};
use nns_governance_canister::types::manage_neuron::ClaimOrRefresh as Cor;
use  nns_governance_canister::types::manage_neuron::claim_or_refresh::{By, MemoAndController};
use nns_governance_canister::types::manage_neuron::Command::ClaimOrRefresh;
use nns_governance_canister::types::manage_neuron::NeuronIdOrSubaccount;
use crate::transaction::transaction::{ITransaction, TransactionCandid};
use crate::transaction::transaction_builder::TransactionBuilder;

impl_basic_for_transaction!(StakeNeuronTransaction);
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StakeNeuronTransaction {
    pub common: BasicTransactionFields,
}

impl StakeNeuronTransaction {
    pub fn new(state: TransactionState, batch_uid: Option<String>, member_id: String, name: String, role: VaultRole) -> Self {
        StakeNeuronTransaction {
            common: BasicTransactionFields::new(state, batch_uid,
                                                true),
        }
    }
}


#[async_trait]
impl ITransaction for StakeNeuronTransaction {
    async fn execute(&mut self, mut state: VaultState) -> VaultState {
        let controller = id();

        let governance_canister_id = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai".to_string()).unwrap();
        let random_bytes = get_random_seed().await;
        let nonce = u64::from_be_bytes(random_bytes[..8].try_into().unwrap());
        let a = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: "dfx_test_key".to_string(),
        };
        // let (response, ): (ManageNeuronResponse, ) = ic_cdk::call(governance_canister_id, "manage_neuron", (mn, )).await
        //     .map_err(|e| format!("Failed to call manage_neuron: {}", e.1)).unwrap();

        if let Ok(public_key) = get_public_key(a).await {
            let subaccount = compute_neuron_staking_subaccount_bytes(get_principal(public_key.clone()).into(), nonce.clone());
            // let b: Subaccount = Subaccount(neuron_subaccount);

            let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
            // let ac = ic_ledger_types::AccountIdentifier::new(&governance_canister_id, &subaccount);
            match icrc_ledger_canister_c2c_client::icrc1_transfer(
                MAINNET_LEDGER_CANISTER_ID,
                &TransferArg {
                    from_subaccount: None,
                    to: Account {
                        owner: governance_canister_id,
                        subaccount: Some(subaccount.clone()),
                    },
                    fee: Some(10_000u32.into()),
                    created_at_time: None,
                    memo: Some(nonce.into()),
                    amount: 100_000_000u32.into(), // 1 ICP
                },
            )
                .await
            {
                Ok(Ok(_)) => {
                    print("transfer success".to_string());
                }
                Ok(Err(error)) => {
                    print(format!("{error:?}"))
                }
                Err(error) => { print(format!("{error:?}")) }
            };

            let mn = ManageNeuron {
                id: None,
                neuron_id_or_subaccount: None,
                command: Some(ClaimOrRefresh(Cor {
                    by: Some(By::MemoAndController(MemoAndController {
                        memo: nonce,
                        controller: Some(get_principal(public_key.clone())),
                    }))
                })),
            };

            let b = prepare_canister_call_via_ecdsa(governance_canister_id,
                                                    "manage_neuron".to_string(), (mn, )).await;
            let response = make_canister_call_via_ecdsa(b).await.unwrap();

            print(format!("response: {:?}", response));

            // let a: NeuronIdOrSubaccount = NeuronIdOrSubaccount::Subaccount(Vec::from(subaccount));
            // print(format!("start c2c call: {:?}", response));
            //
            // let (response, ): (Neuron, ) = ic_cdk::call(governance_canister_id, "get_full_neuron_by_id_or_subaccount", (a, )).await
            //     .map_err(|e| format!("Failed to call get_full_neuron_by_id_or_subaccount: {}", e.1)).unwrap();

            match nns_governance_canister_c2c_client::manage_neuron(
                governance_canister_id,
                &ManageNeuron {
                    id: None,
                    neuron_id_or_subaccount: None,
                    command: Some(ClaimOrRefresh(nns_governance_canister::types::manage_neuron::ClaimOrRefresh {
                        by: Some(By::MemoAndController(MemoAndController {
                            controller: Some(get_principal(public_key.clone()).into()),
                            memo: nonce.clone(),
                        })),
                    })),
                },
            )
                .await
            {
                Ok(response) => match response.command {
                    Some(manage_neuron_response::Command::ClaimOrRefresh(c)) => {
                        let neuron_id = c.refreshed_neuron_id.unwrap().id;
                        print("Staked new NNS neuron".to_string());
                        print(neuron_id.to_string())
                    }
                    response => {
                        error!(?response, "Governance error");
                        print(format!("{response:?}"))
                    }
                },
                Err(error) => print(format!("{error:?}")),
            }
            // print(format!("finis c2c call: {:?}", match response.controller {
            //     None => { "NOTHING".to_string()}
            //     Some(s) => {s.to_text()}
            // }));
        }
        let neurons = nns_governance_canister_c2c_client::list_neurons(governance_canister_id, &nns_governance_canister::types::ListNeurons {
            neuron_ids: Vec::new(),
            include_neurons_readable_by_caller: true,
        }, ).await.unwrap();

        let len = neurons.full_neurons.len();
        print(len.to_string());

        for neuron in neurons.full_neurons {
            print(format!("neuron: {:?}", neuron.id.unwrap().id));
        }

        // let ln_call = prepare_canister_call_via_ecdsa(governance_canister_id,
        //                                         "list_neurons".to_string(), args).await;
        //
        // let response = make_canister_call_via_ecdsa(ln_call).await.unwrap();
        //
        // print(format!("response 222: {:?}", response));

        state
    }

    fn to_candid(&self) -> TransactionCandid {
        let trs: StakeNeuronTransaction = self.clone();
        TransactionCandid::StakeNeuronTransactionV(trs)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct StakeNeuronTransactionRequest {
    role: VaultRole,
    batch_uid: Option<String>,
}

pub struct StakeNeuronTransactionBuilder {
    request: StakeNeuronTransactionRequest,
}

impl StakeNeuronTransactionBuilder {
    pub fn init(request: StakeNeuronTransactionRequest) -> Self {
        return StakeNeuronTransactionBuilder {
            request
        };
    }
}

impl TransactionBuilder for StakeNeuronTransactionBuilder {
    async fn build_dyn_transaction(&mut self, state: TransactionState) -> Box<dyn ITransaction> {
        let trs = StakeNeuronTransaction::new(
            state,
            self.request.batch_uid.clone(),
            "".to_string(),
            "".to_string(),
            self.request.role,
        );
        Box::new(trs)
    }
}


#[test]
fn sub_account_test() {}

pub fn now_nanos() -> u64 {
    if std::env::var("TEST_FIXED_TIMESTAMP").is_ok() {
        1_669_073_904_187_044_208
    } else {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}


// We currently only support a subset of the functionality.
pub async fn exec(pem: &Option<String>) {
    let args = Encode!(&ListNeurons {
        neuron_ids: Vec::new(),
        include_neurons_readable_by_caller: true,
    });
}

// async fn aaaa(mn: ManageNeuron, governance_canister_id: Principal) -> ManageNeuronResponse {
//     let (response,): (ManageNeuronResponse,) = ic_cdk::call(governance_canister_id, "manage_neuron", (mn, )).await
//         .map_err(|e| format!("Failed to call manage_neuron: {}", e.1))?;
//     // print(format!("response: {:?}", response.command.));
//     response
//     // Ok(match response.command {
//     //     Some(manage_neuron_response::Command::ClaimOrRefresh(_)) => Ok(()),
//     //     Some(manage_neuron_response::Command::Error(error)) => Err(error),
//     //     Some(_) => unreachable!(),
//     //     None => {
//     //         // This will be reached if we fail to deserialize the response
//     //         // TODO remove this arm once candid is fixed (if ever).
//     //         print("Failed to deserialize response".to_string());
//     //         Ok(())
//     //     }
//     // })
// }

// Get a random seed based on 'raw_rand'
pub async fn get_random_seed() -> [u8; 32] {
    let raw_rand = match management_canister::main::raw_rand().await {
        Ok((res, )) => res,
        Err((_, err)) => ic_cdk::trap(&format!("failed to get seed: {err}")),
    };

    raw_rand.as_slice().try_into().unwrap_or_else(|_| {
        ic_cdk::trap(&format!(
            "when creating seed from raw_rand output, expected raw randomness to be of length 32, got {}",
            raw_rand.len()
        ));
    })
}
