#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx deploy vault --argument "(opt record { ledger_canister_id=principal \"${LEDGER_ID}\";}, )"

echo "DONE"
