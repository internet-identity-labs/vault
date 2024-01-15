#!/usr/bin/env bash
echo "===========DEPLOY VAULT MANAGER========="
dfx identity use test_admin

pwd

dfx deploy vault_manager --argument '(record { origins = vec {}; initial_cycles_balance = 500_000_000_000 : nat; payment_cycles = 100_000_000 : nat64; respo_canister_id = "7jlkn-paaaa-aaaap-abvpa-cai"; destination_address = "4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e" } )'

echo "DONE"
