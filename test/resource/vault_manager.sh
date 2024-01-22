#!/usr/bin/env bash
echo "===========DEPLOY VAULT MANAGER========="
dfx identity use test_admin

pwd

dfx deploy vault_manager --specified-id=sgk26-7yaaa-aaaan-qaovq-cai --argument '(record { origins = vec {"http://localhost:4200";"https://vaults-dev.nfid.one";https://hoj3i-aiaaa-aaaak-qcl7a-cai.icp0.io";}; initial_cycles_balance = 500_000_000_000 : nat; payment_cycles = 100_000_000 : nat64; repo_canister_id = "7jlkn-paaaa-aaaap-abvpa-cai"; destination_address = "4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e" } )'

echo "DONE"
