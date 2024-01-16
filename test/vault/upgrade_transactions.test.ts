import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory as vrIdl} from "./../vault_repo/sdk/vr_idl";
import {
    VaultManager,
} from "./sdk_prototype/vault_manager";
import {fromHexString, principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {getTransactionByIdFromGetAllTrs, requestVersionUpgradeTransaction, verifyTransaction} from "./helper";
import {readWasmFile} from "../vault_repo/vault_repo.test";
import {sha256} from "ethers/lib/utils";
import {VaultWasm} from "../vault_repo/sdk/vr";
import {createCanister} from "../vault_manager/sdk/ochestrator";
import {TransactionState, TransactionType} from "./sdk_prototype/enums";
import {VersionUpgradeTransaction} from "./sdk_prototype/transactions";
import {Approve} from "./sdk_prototype/approve";

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
        console.log(execute(`./test/resource/ledger.sh`))

        console.log(execute(`dfx deploy vault_repo  --argument '(record { controllers = vec {}; origins = vec {}; })' --specified-id=7jlkn-paaaa-aaaap-abvpa-cai`))
        vault_repo_id = DFX.GET_CANISTER_ID("vault_repo");
        DFX.ADD_CONTROLLER(admin_identity.getPrincipal().toText(), vault_repo_id);
        await console.log(execute(`dfx canister call vault_repo sync_controllers`))
        let wasm_bytes = readWasmFile("test/vault_repo/vault_001.wasm");
        let actor = await getActor(vault_repo_id, admin, vrIdl);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
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
        vault_canister_id = await createCanister(vault_manager_canister, admin, BigInt(1));
        manager = new VaultManager()
        await manager.init(vault_canister_id, admin, true)
        let state = await manager.getState()
        expect(state.members.length).eq(1);

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
        let wasm_bytes = readWasmFile("test/vault_repo/vault_002.wasm");
        let actor = await getActor(vault_repo_id, admin, vrIdl);
        let hash = sha256(wasm_bytes);
        let wasm: VaultWasm = {
            wasm_module: Array.from(wasm_bytes),
            hash: hash,
            version: "0.0.2"
        }
        await actor.add_version(wasm);
        let trRequestResponse = await requestVersionUpgradeTransaction(manager, "0.0.2");
        let trs = trRequestResponse[0];
        try {
            await manager.execute();
        }catch (e) {
            console.log(e)
        }
        let tr = await getTransactionByIdFromGetAllTrs(manager, trs.id);
        expect(tr.state).eq(TransactionState.Executed)
        let version = await manager.getVersion();
        expect(version).eq("0.0.2")
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

