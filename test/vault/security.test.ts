import {DFX} from "../constanst/dfx.const";
import {getIdentity} from "../util/deployment.util";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {expect} from "chai";
import {fail} from "assert";
import {requestCreateMemberTransaction, requestUpdateQuorumTransaction} from "./helper";
import {TransactionState, VaultManager, VaultRole} from "@nfid/vaults";

require('./bigintextension.js');

describe("Security ", () => {
    let canister_id;
    const admin = getIdentity("87654321876543218765432187654321");
    const admin2 = getIdentity("87654321876543218765432187654323");
    const member = getIdentity("87654321876543218765432187654320");
    const notRegistered = getIdentity("87654321876543218765432187654328");
    let adminManager: VaultManager;
    let adminManager2: VaultManager;
    let memberManager: VaultManager;
    let notRegisteredManager: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        adminManager = new VaultManager(canister_id, admin);
        adminManager2 = new VaultManager(canister_id, admin2);
        memberManager = new VaultManager(canister_id, member);
        notRegisteredManager = new VaultManager(canister_id, notRegistered);
        await adminManager.resetToLocalEnv();
        await adminManager2.resetToLocalEnv();
        await memberManager.resetToLocalEnv();
        await notRegisteredManager.resetToLocalEnv();
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
            await memberManager.approveTransaction([{trId: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Not permitted")
        }
    });
    it("Approve state transaction with admin role - have to be approved", async function () {
        try {
            await adminManager.approveTransaction([{trId: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Already approved")
        }
    });
    it("Approve state transaction with admin role - have to be approved", async function () {
        try {
            await adminManager2.approveTransaction([{trId: id, state: TransactionState.Approved}])
            await memberManager.execute()
            await adminManager2.approveTransaction([{trId: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Transaction is immutable")
        }
    });

    it("Request register transaction from the notregistered", async function () {
        try {
            await requestCreateMemberTransaction(notRegisteredManager, "testAddress2", "testName", VaultRole.MEMBER)
            fail("")
        } catch (e) {
            expect(e.message).contains("Not registered")
        }
    });

    it("Request approve transaction from the notregistered", async function () {
        let t = await requestCreateMemberTransaction(adminManager, "testAddress2", "testName", VaultRole.MEMBER)
        id = t[0].id;
        try {
            await notRegisteredManager.approveTransaction([{trId: id, state: TransactionState.Approved}])
            fail("")
        } catch (e) {
            expect(e.message).contains("Not registered")
        }
    });

    //TODO transfer from member


})
