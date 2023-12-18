#!/usr/bin/env bash
echo "===========DEPLOY VAULT MANAGER========="
dfx identity use test_admin

pwd

dfx deploy vault_manager
#dfx deploy vault

echo "DONE"
