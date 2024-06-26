import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute, sleep} from "../util/call.util";
import {getTransactionByIdFromGetAllTrs, requestUpdateVaultNamingTransaction, verifyTransaction} from "./helper";
import {Approve, TransactionState, TransactionType, VaultManager} from "@nfid/vaults";
import {Transaction} from "@nfid/vaults";
import {VaultUpdateNamingTransaction} from "@nfid/vaults";
import {QuorumUpdateTransaction} from "@nfid/vaults";

require('./bigintextension.js');

describe("Vault Naming Transactions", () => {
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    let manager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        const admin = getIdentity("87654321876543218765432187654321");
        canister_id = DFX.GET_CANISTER_ID("vault");
        manager = new VaultManager(canister_id, admin_identity);
        await manager.resetToLocalEnv();
    });

    after(() => {
        DFX.STOP();
    });

    it("UpdateVault naming", async function () {
        let trReqResp: Array<Transaction> = await requestUpdateVaultNamingTransaction(manager, "Name", "Description")
        let trId = trReqResp[0].id
        await sleep(2);
        let tr = await getTransactionByIdFromGetAllTrs(manager, trId)
        let expectedTrs: VaultUpdateNamingTransaction = buildExpectedNamingTransaction(TransactionState.Executed)
        verifyUpdateVaultNamingTransaction(expectedTrs, tr as QuorumUpdateTransaction)
        let state = await manager.getState();
        expect(state.name).eq("Name")
        expect(state.description).eq("Description")
    });

    function buildExpectedNamingTransaction(state) {
        let expectedApprove: Approve = {
            createdDate: 0n,
            signer: principalToAddress(admin_identity.getPrincipal() as any),
            status: TransactionState.Approved
        }
        let expectedTrs: VaultUpdateNamingTransaction = {
            modifiedDate: 0n,
            name: "Name",
            description: "Description",
            threshold: 1,
            approves: [expectedApprove],
            batchUid: undefined,
            createdDate: 0n,
            id: 0n,
            initiator: principalToAddress(admin_identity.getPrincipal() as any),
            isVaultState: true,
            state,
            transactionType: TransactionType.VaultNamingUpdate
        }
        return expectedTrs
    }

})

function verifyUpdateVaultNamingTransaction(expected: VaultUpdateNamingTransaction, actual: VaultUpdateNamingTransaction) {
    expect(expected.name).eq(actual.name)
    expect(expected.description).eq(actual.description)
    verifyTransaction(expected, actual, TransactionType.VaultNamingUpdate)
}

