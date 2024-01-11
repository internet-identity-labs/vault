import { call, execute } from "../util/call.util"

export const DFX = {
    STOP: () => execute(`dfx stop; kill -9 $(lsof -i TCP:8000 | grep LISTEN | awk '{print $2}')`),
    REMOVE_DFX_FOLDER: () => execute(`rm -rf .dfx`),
    CREATE_TEST_PERSON: () => execute(`dfx identity new test`),
    USE_TEST_ADMIN: () => execute(`dfx identity use test_admin`),
    SYNC_CONTROLLERS: () => execute(`dfx canister call vault sync_controllers`),
    GET_PRINCIPAL: () => call(`dfx identity get-principal`),
    INIT: () => execute(`dfx start --clean --background`),
    UPGRADE_FORCE: (x: string) => execute(`dfx canister install --mode upgrade --upgrade-unchanged ${x} `),
    GET_CANISTER_ID: (x: string) => call(`dfx canister id ${x}`),
    ADD_CONTROLLER: (x: string, y: string) => execute(`dfx canister update-settings --add-controller "${x}" ${y}`),
    LEDGER_FILL_BALANCE: (x:string) => call(`dfx canister call ledger transfer "(record { to=vec { ${x} };
          amount=record { e8s=100_000_000 }; fee=record { e8s=10_000 : nat64 }; memo=0:nat64; } )"`),
}