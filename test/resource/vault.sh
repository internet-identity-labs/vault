#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx identity use test_admin

#dfx deploy vault --argument "(opt record { ledger_canister_id=principal \"${LEDGER_ID}\"; is_test_env = opt true }, )"
dfx deploy vault

echo "DONE"
