# NFID Vaults

This repository represents a protocol for creating and managing Vault products. It consists of 3 sub-projects:

- **Vault**: A self-sovereign protocol, serving as a smart contract for managing access to canister signature of the Internet Computer protocol.
- **VaultManager**: A smart contract for creating Vault objects.
- **VaultRepository**: A smart contract for storing versions (WASM) of Vault products.

## Vault

To deploy a local vault canister, run the following command:

```bash
dfx deploy vault --argument  '(principal "INITIAL_VAULT_ADMIN", record { origins = vec {}; repo_canister = "VAULT_REPO_CANISTER_ID" })'
```
Where:

- `INITIAL_VAULT_ADMIN` is the principal of the initial admin of the vault.
- `VAULT_REPO_CANISTER_ID` is the ID of the vault repository canister that manages upgrades of the vault version.

## VaultManager

VaultManager allows the creation of vault canisters. To deploy a local vault manager canister, run the following command:

```bash
dfx deploy vault_manager  --argument '(record { origins = vec {"TRUSTED_ORIGIN";}; initial_cycles_balance = INITILA_CYCLE_BALANCE : nat; icp_price = E8S_PRICE : nat64; repo_canister_id = "VAULT_REPO_CANISTER_ID"; destination_address = "PROTOCOL_WALLET_ADDRESS" } )'
```

- `TRUSTED_ORIGIN` is the list of origins that can call the vault manager canister.
- `INITILA_CYCLE_BALANCE` is the initial cycles balance of the created vault canister.
- `E8S_PRICE` is the protocol price of vault creation.
- `VAULT_REPO_CANISTER_ID` is the ID of the vault repository canister that manages upgrades of the vault version.
- `PROTOCOL_WALLET_ADDRESS` is the address of the protocol wallet that receives the protocol fee.

To create a Vault canister through the VaultManager, run the following method with the specified arguments:

- `TRANSACTION_ID` is the transaction ID in the ledger to verify that `E8S_PRICE` is paid.
- `Pro` or `Light` is the type of the vault canister to create (default is `Pro`).
- `VAULT_ADMIN_PRINCIPAL` is the principal of the admin of the newly created vault canister.
## VaultRepository

VaultRepository allows storing and managing versions of vault canisters.
```bash
dfx deploy vault_repo  --argument '(record { controllers = vec {}; origins = vec {"http://localhost:4200";}; })'
```
- `controllers` is the list of principals that can manage the vault repository canister. You can leave it empty if you want to and fill it with the `sync_controllers` interface later.
- `origins` is the list of origins that can call the vault repository canister.

### Tests

To run tests, execute the following command:

```bash
npm test
```

### Local deployment:

TBD

You can also find an example of how to run whole working environment in `vault_manager.test.ts`.

