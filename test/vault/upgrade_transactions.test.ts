import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory as vrIdl} from "./../vault_repo/sdk/vr_idl";
import {fromHexString, principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateWalletTransaction,
    requestVersionUpgradeTransaction,
    verifyTransaction
} from "./helper";
import {readWasmFile} from "../vault_repo/vault_repo.test";
import {sha256} from "ethers/lib/utils";
import {VaultWasm} from "../vault_repo/sdk/vr";
import {createCanister} from "../vault_manager/sdk/ochestrator";
import {Approve, Network, TransactionState, TransactionType, VaultManager} from "@nfid/vaults";
import {VersionUpgradeTransaction} from "@nfid/vaults";

require('./bigintextension.js');

describe("Upgrade Transactions", () => {
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    let vault_canister_id: string;
    let vault_repo_id: string;
    let vault_manager_canister: string
    const admin = getIdentity("87654321876543218765432187654321");

    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        console.log(execute(`dfx deploy vault_repo  --argument '(record { controllers = vec {}; origins = vec {}; })' --specified-id=7jlkn-paaaa-aaaap-abvpa-cai`))
        vault_repo_id = DFX.GET_CANISTER_ID("vault_repo");
        DFX.ADD_CONTROLLER(admin_identity.getPrincipal().toText(), vault_repo_id);
        await console.log(execute(`dfx canister call vault_repo sync_controllers`))
        let wasm_bytes = readWasmFile("test/vault_repo/vault_001.wasm");
        let actor = await getActor(vault_repo_id, admin, vrIdl);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            description: [],
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.1"
        }
        await actor.add_version(wasm);
        console.log(execute(`./test/resource/ledger.sh`))
        let correctBytes = fromHexString("4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e")
        console.log(DFX.LEDGER_FILL_BALANCE(correctBytes.toString().replaceAll(',', ';')))
        console.log(execute(`./test/resource/vault_manager.sh`))
        vault_manager_canister = DFX.GET_CANISTER_ID("vault_manager");
        vault_canister_id = await createCanister(vault_manager_canister, admin, BigInt(1), []);
        manager = new VaultManager(vault_canister_id, admin)
        await manager.resetToLocalEnv()
        await requestCreateWalletTransaction(manager, "walletName", Network.IC);
        await requestCreateWalletTransaction(manager, "walletName2", Network.IC);
        await manager.execute();
        let state = await manager.getState()
        expect(state.members.length).eq(1);
        expect(state.wallets.length).eq(2);
    });

    after(() => {
        DFX.STOP();
    });

    it("Upgrade approved and rejected", async function () {
        let trRequestResponse = await requestVersionUpgradeTransaction(manager, "0.0.1");
        let trs = trRequestResponse[0];
        await manager.execute();
        let tr = await getTransactionByIdFromGetAllTrs(manager, trs.id);
        let expected = buildExpectedVersionUpgradeTransaction(TransactionState.Rejected)
        verifyUpgradeTransaction(expected, tr)
    });

    it("Upgrade approved and executed", async function () {
        //build the latest version and add it to the vault repo
        await console.log(execute(`./test/resource/vault.sh`))
        let canister_id = DFX.GET_CANISTER_ID("vault");
        let vm = new VaultManager(canister_id, admin_identity);
        await vm.resetToLocalEnv();
        let latestVaultVersion = await vm.getVersion();

        let wasm_bytes = readWasmFile(".dfx/local/canisters/vault/vault.wasm");
        let actor = await getActor(vault_repo_id, admin, vrIdl);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            description: [],
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: latestVaultVersion
        }
        await actor.add_version(wasm);
        let trRequestResponse = await requestVersionUpgradeTransaction(manager, latestVaultVersion);
        let trs = trRequestResponse[0];
        try {
            await manager.execute();
        }catch (e) {
            console.log(e)
        }
        await manager.execute();
        let tr = await getTransactionByIdFromGetAllTrs(manager, trs.id) as VersionUpgradeTransaction;
        expect(tr.state).eq(TransactionState.Executed)
        expect(tr.initial_version).eq("0.0.1")
        let version = await manager.getVersion();
        expect(version).eq(latestVaultVersion)
        let state = await manager.getState()
        expect(state.members.length).eq(1);
        expect(state.wallets.length).eq(2);
    });


    it("Upgrade approved and rejected with smaller version", async function () {
        let trRequestResponse = await requestVersionUpgradeTransaction(manager, "0.0.1");
        let trs = trRequestResponse[0];
        await manager.execute();
        let tr = await getTransactionByIdFromGetAllTrs(manager, trs.id);
        expect(tr.state).eq(TransactionState.Rejected)
    });

    function buildExpectedVersionUpgradeTransaction(state) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }

        let expectedTrs: VersionUpgradeTransaction = {
            version: "0.0.1",
            initial_version: "0.0.1",
            modifiedDate: 0n,
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.WalletCreate
        }
        return expectedTrs
    }
})

export function verifyUpgradeTransaction(expected: VersionUpgradeTransaction, actual: VersionUpgradeTransaction) {
    expect(expected.version).eq(actual.version)
    verifyTransaction(expected, actual, TransactionType.WalletCreate)
}

