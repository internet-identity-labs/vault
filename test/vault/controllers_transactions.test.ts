import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory} from "./sdk_prototype/idl";
import {ActorMethod} from "@dfinity/agent";
import {VaultManager,} from "./sdk_prototype/vault_manager";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {execute} from "../util/call.util";
import {
    getTransactionByIdFromGetAllTrs,
    requestCreateMemberTransaction,
    requestUpdateControllersTransaction,
    requestUpdateQuorumTransaction
} from "./helper";
import {TransactionState, VaultRole} from "./sdk_prototype/enums";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {ApproveRequest} from "./sdk_prototype/approve";
import {fail} from "assert";
import {Principal} from "@dfinity/principal";

require('./bigintextension.js');

describe("Controller Transactions", () => {
    let admin_actor_1: Record<string, ActorMethod>;
    let member_actor_1: Record<string, ActorMethod>;
    let canister_id;
    let admin_identity = getIdentity("87654321876543218765432187654321")
    const member = getIdentity("87654321876543218765432187654320");
    let manager: VaultManager;
    let manager2: VaultManager;
    before(async () => {
        DFX.INIT();
        DFX.USE_TEST_ADMIN();
        await console.log(execute(`./test/resource/vault.sh`))
        canister_id = DFX.GET_CANISTER_ID("vault");
        admin_actor_1 = await getActor(canister_id, admin_identity, idlFactory);
        member_actor_1 = await getActor(canister_id, member, idlFactory);
        manager = new VaultManager();
        manager2 = new VaultManager();
        await manager.init(canister_id, admin_identity, true);
        await manager2.init(canister_id, member, true);
    });

    after(() => {
        DFX.STOP();
    });

    it("Controllers rejects transactions because canister not controller but transaction not blocked by queue", async function () {
        await requestCreateMemberTransaction(manager, principalToAddress(member.getPrincipal() as any), "memberName", VaultRole.ADMIN)
        await manager.execute()
        await requestUpdateQuorumTransaction(manager, 2)
        await manager.execute();
        let controllers = await manager.getControllers();
        let principal1 = Ed25519KeyIdentity.generate().getPrincipal();
        let principal2 = Ed25519KeyIdentity.generate().getPrincipal();
        await requestCreateMemberTransaction(manager, principalToAddress(principal1 as any), "memberName", VaultRole.ADMIN)
        await requestCreateMemberTransaction(manager, principalToAddress(principal2 as any), "memberName", VaultRole.ADMIN)
        try {
            await requestUpdateControllersTransaction(manager, [principal1, principal2, ])
            fail("Should throw error")
        }catch (e) {
            expect(e.message).contains("The Vault canister needs to be included in the list of controllers to enable self-updating.")
        }
        let tr = await requestUpdateControllersTransaction(manager, [principal1, principal2, Principal.fromText(canister_id)])
        let approveRequest: ApproveRequest = {
            trId: tr[0].id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approveRequest])
        await manager.execute();
        let rejectedTransaction = await getTransactionByIdFromGetAllTrs(manager, tr[0].id);
        expect(rejectedTransaction.state).eq(TransactionState.Rejected);
    });

    it("Add canister self as a controller and execute", async function () {
        await console.log(execute(`dfx canister update-settings vault --add-controller ` + canister_id))
        let principal1 = Ed25519KeyIdentity.generate().getPrincipal();
        let principal2 = member.getPrincipal();
        let tr = await requestUpdateControllersTransaction(manager, [principal1, principal2, Principal.fromText(canister_id)])
        let approveRequest: ApproveRequest = {
            trId: tr[0].id,
            state: TransactionState.Approved
        }
        await manager2.approveTransaction([approveRequest])
        await manager.execute();
        let controllers = await manager.getControllers();
        let actualPrincipals = controllers.map(c => c.toText());
        expect(actualPrincipals).contains(principal1.toText());
        expect(actualPrincipals).contains(principal2.toText());
        expect(actualPrincipals).contains(canister_id);
    });

})

