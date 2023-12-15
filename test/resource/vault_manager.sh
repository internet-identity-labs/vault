#!/usr/bin/env bash
echo "===========DEPLOY VAULT MANAGEr========="
dfx identity use test_admin

pwd

dfx deploy vault_manager
#dfx deploy vault

echo "DONE"
