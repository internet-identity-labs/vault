#!/usr/bin/env bash
echo "===========SETUP========="
export IC_VERSION=a17247bd86c7aa4e87742bf74d108614580f216d
gunzip ledger.wasm.gz
test -f ledger.wasm.gz ||curl -o ledger.wasm.gz "wget https://download.dfinity.systems/ic/a17247bd86c7aa4e87742bf74d108614580f216d/canisters/ledger-canister.wasm.gz"
test -f ledger.wasm || gunzip ledger.wasm.gz
test -f ledger.did || curl -o ledger.did "https://raw.githubusercontent.com/dfinity/ic/a17247bd86c7aa4e87742bf74d108614580f216d/rs/rosetta-api/icp_ledger/ledger.did"


echo "===========START DFX========="
dfx start --background --clean
cat <<<"$(jq '.canisters.ledger.candid="ledger.did"' dfx.json)" >dfx.json
export MINT_ACC=$(dfx --identity anonymous ledger account-id)
export LEDGER_ACC=$(dfx ledger account-id)
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)
echo "===========DEPLOY LEDGER========="
#dfx deploy ledger --argument '(record {minting_account = "'${MINT_ACC}'"; initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; }; send_whitelist = vec {}})'


dfx deploy ledger --argument "(variant {
  Init = record {
    minting_account = \"$MINT_ACC\";
    icrc1_minting_account = opt record {
      owner = principal \"$MINT_ACC\";
      subaccount = null;
    };
    initial_values = vec {
      record {
        \"$MINT_ACC\";
        record {
          e8s = 100000000 : nat64;
        };
      };
    };
    max_message_size_bytes = opt(2560000 : nat64);
    transaction_window = opt record {
      secs = 10 : nat64;
      nanos = 0 : nat32;
    };
    archive_options = opt record {
      trigger_threshold = 1000000 : nat64;
      num_blocks_to_archive = 1000000 : nat64;
      node_max_memory_size_bytes = null;
      max_message_size_bytes = null;
      controller_id = principal \"$MINT_ACC\";
      cycles_for_archive_creation = null;
    };
    send_whitelist = vec {
      principal \"$MINT_ACC\";
    };
    transfer_fee = opt record {
      e8s = 1000000 : nat64;
    };
    token_symbol =  \"ICP\";
    token_name = \"ICP\";
  }})"
cat <<<"$(jq '.canisters.ledger.candid="ledger.did"' dfx.json)" >dfx.json

