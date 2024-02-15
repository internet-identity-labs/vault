#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx identity use test_admin

dfx deploy vault --argument '(principal "3ekng-5nqql-esu4u-64sla-pcm5o-hjatn-hwjo7-vk7ya-ianug-zqqyy-iae", record { origins = vec {}; repo_canister = "7jlkn-paaaa-aaaap-abvpa-cai" })'

echo "DONE"
