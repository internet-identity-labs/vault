#!/usr/bin/env bash
echo "===========SETUP========="
test -f ledger.wasm.gz || curl -o ledger.wasm.gz "https://download.dfinity.systems/ic/d87954601e4b22972899e9957e800406a0a6b929/canisters/ledger-canister.wasm.gz"
test -f ledger.wasm || gunzip ledger.wasm.gz
test -f ledger.did || curl -o ledger.did "https://raw.githubusercontent.com/dfinity/ic/d87954601e4b22972899e9957e800406a0a6b929/rs/rosetta-api/icp_ledger/ledger.did"

dfx identity use default

export MINT_ACC=$(dfx --identity anonymous ledger account-id)
export LEDGER_ACC=$(dfx --identity test_admin ledger account-id)
dfx identity use test_admin

echo $MINT_ACC
echo $(dfx identity get-principal)
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)
echo "===========DEPLOY LEDGER========="

dfx deploy ledger --specified-id=ryjl3-tyaaa-aaaaa-aaaba-cai --argument "
  (variant {
    Init = record {
      minting_account = \"$MINT_ACC\";
      initial_values = vec {
        record {
          \"$LEDGER_ACC\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"ICP\";
      token_name = opt \"Local ICP\";
    }
  })
"

