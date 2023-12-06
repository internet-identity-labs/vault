import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {TransactionState, VaultManager, VaultRole} from "./sdk_prototype/vault_manager";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {fail} from "assert";
import {requestCreateMemberTransaction, requestUpdateQuorumTransaction} from "./helper";

require('./bigintExtension.js');

describe("Security ", () => {
    let canister_id;
    const admin = getIdentity("87654321876543218765432187654321");
    const admin2 = getIdentity("87654321876543218765432187654323");
    let adminManager: VaultManager;
    let adminManager2: VaultManager;
    let memberManager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/ledger.sh`))
        await console.log(execute(`./test/resource/vault.sh`))
        const member = getIdentity("87654321876543218765432187654320");
        canister_id = DFX.GET_CANISTER_ID("vault");
        adminManager = new VaultManager();
        adminManager2 = new VaultManager();
        memberManager = new VaultManager();
        await adminManager.init(canister_id, admin, true);
        await adminManager2.init(canister_id, admin2, true);
        await memberManager.init(canister_id, member, true);
        await adminManager.execute()
        await requestCreateMemberTransaction(adminManager, principalToAddress(admin2.getPrincipal() as any), "Admin2", VaultRole.ADMIN)
        await requestCreateMemberTransaction(adminManager, principalToAddress(member.getPrincipal() as any), "Member", VaultRole.MEMBER)
        await requestUpdateQuorumTransaction(adminManager, 2)
        await adminManager.execute()
    });

    after(() => {
        DFX.STOP();
    });

    it("Request vault state transaction from the member", async function () {
        try {
            await requestCreateMemberTransaction(memberManager, "testAddress", "testName", VaultRole.MEMBER)
            fail("")
        } catch (e) {
            expect(e.message).contains("Not permitted")
        }
    });
    let id;
    it("Request vault state transaction from the admin but approve from member", async function () {
        let t = await requestCreateMemberTransaction(adminManager, "testAddress", "testName", VaultRole.MEMBER)
        id = t[0].id;
        try {
            await memberManager.approveTransaction([{tr_id: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Not permitted")
        }
    });
    it("Approve state transaction with admin role - have to be approved", async function () {
        try {
            await adminManager.approveTransaction([{tr_id: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Already approved")
        }
    });
    it("Approve state transaction with admin role - have to be approved", async function () {
        try {
            await adminManager2.approveTransaction([{tr_id: id, state: TransactionState.Approved}])
            await memberManager.execute()
            await adminManager2.approveTransaction([{tr_id: id, state: TransactionState.Approved}])
        } catch (e) {
            expect(e.message).contains("Transaction is immutable")
        }
    });

    //TODO transfer from member


})
