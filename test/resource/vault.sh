#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx identity use test_admin

dfx deploy vault --argument "(opt record { ledger_canister_id=principal \"${LEDGER_ID}\"}, )"
#dfx deploy vault

echo "DONE"
