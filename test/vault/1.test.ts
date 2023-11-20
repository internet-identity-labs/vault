import {DFX} from "../constanst/dfx.const";
import {getActor, getIdentity} from "../util/deployment.util";
import {idlFactory} from "./sdk_prototype/idl";
import {ActorMethod} from "@dfinity/agent";
import {
    TransactionApproveRequest,
    TransactionCandid,
    TransactionRequest,
    VaultState
} from "./sdk_prototype/service_vault";
import {
    ApproveRequest,
    MemberCreateTransactionRequest,
    QuorumTransactionRequest, Transaction,
    TransactionState,
    VaultManager,
    VaultRole
} from "./sdk_prototype/vault_manager";

require('./bigintExtension.js');

describe("PPPPPP", () => {
    let admin_actor_1: Record<string, ActorMethod>;
    let member_actor_1: Record<string, ActorMethod>;
    let canister_id = "bkyz2-fmaaa-aaaaa-qaaaq-cai"

    before(async () => {
        DFX.USE_TEST_ADMIN();
        // await console.log(execute(`./test/resource/ledger.sh`))
        // await console.log(execute(`./test/resource/vault.sh`))
        const admin = getIdentity("87654321876543218765432187654321");
        const member = getIdentity("87654321876543218765432187654320");
        // const vault_id = DFX.GET_CANISTER_ID("vault");

        admin_actor_1 = await getActor(canister_id, admin, idlFactory);
        member_actor_1 = await getActor(canister_id, member, idlFactory);

    });

    after(() => {
        // DFX.STOP();
    });

    it("register123123", async function () {

        let manager = new VaultManager();
        await manager.init(canister_id, getIdentity("87654321876543218765432187654321"), true);


       let quorumR = new QuorumTransactionRequest(2);
       let memberR = new MemberCreateTransactionRequest("testId", "Ololo", VaultRole.MEMBER);

        let ttttttrs = await manager.requestTransaction([memberR, quorumR])
        let manager_trs = await manager.getTransactions();

        let vault1 = await manager.getState();
        let state = await admin_actor_1.get_state([]) as VaultState;
        let app: ApproveRequest= {
            state: TransactionState.Approved, tr_id: 1n

        }
        let app2: ApproveRequest= {
            state: TransactionState.Approved, tr_id: 2n

        }
       let k = await  manager.approveTransaction([app, app2]) as Array<Transaction>
           await  admin_actor_1.execute()
        let manager_trs2 = await manager.getTransactions();

        let vault2 = await manager.redefineState();

        let vault3 = await manager.redefineState(0n);

        let vault4 = manager.getState()
        let state3 = await admin_actor_1.get_state([]) as VaultState;

        console.log("")


    });

})
