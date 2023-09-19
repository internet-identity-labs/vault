import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {
    Approve,
    Policy,
    PolicyRegisterRequest,
    ThresholdPolicy,
    Transaction,
    Vault,
    VaultMemberRequest,
    VaultRegisterRequest,
    Wallet
} from "../idl/vault";
import {expect} from "chai";
import {fromHexString, principalToAddress, principalToAddressBytes} from "ictool"
import {DFX} from "../constanst/dfx.const";
import {Principal} from "@dfinity/principal";
import {fail} from "assert";


describe.skip("Transaction", () => {
    var dfx: Dfx;
    let adminAddress: string;
    let memberAddress1: string;
    let memberAddress2: string;
    let vault1: Vault;
    let vault2: Vault;
    let vault3: Vault;
    let wallet1: Wallet;
    let wallet2: Wallet;
    let wallet3: Wallet;
    let policy: Policy;
    let policy2: Policy;
    let to: string;
    let tokens = 100000n

    before(async () => {
        dfx = await deploy({apps: [App.Vault]});

        adminAddress = principalToAddress(dfx.user.identity.getPrincipal() as any);
        memberAddress1 = principalToAddress(dfx.vault.member_1.getPrincipal() as any);
        memberAddress2 = principalToAddress(dfx.vault.member_2.getPrincipal() as any);

        let request: VaultRegisterRequest = {
            description: ["test"],
            name: "vault1"
        };
        vault1 = await dfx.vault.admin_actor.register_vault(request) as Vault
        vault2 = await dfx.vault.admin_actor.register_vault(request) as Vault

        let vaultMember: VaultMemberRequest = {
            address: memberAddress1,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n,
            state: {'Active': null},
        }

        await dfx.vault.admin_actor.store_member(vaultMember) as Vault;
        vaultMember.address = memberAddress2;
        await dfx.vault.admin_actor.store_member(vaultMember) as Vault;
        wallet1 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet1"], vault_id: 1n}) as Wallet
        let walBytes = principalToAddressBytes(Principal.fromText(dfx.vault.id) as any, fromHexString(wallet1.uid))
        DFX.LEDGER_FILL_BALANCE(walBytes.toString().replaceAll(',', ';'))
        wallet2 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
        let tp: ThresholdPolicy = {
            amount_threshold: 1n,
            currency: {'ICP': null},
            member_threshold: [1],
            wallets: []
        }
        let request1: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 1n};
        policy = await dfx.vault.admin_actor.register_policy(request1) as Policy
        let tp2: ThresholdPolicy = {
            amount_threshold: 1n,
            currency: {'ICP': null},
            member_threshold: [1],
            wallets: []
        }
        let request2: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 2n};
        policy2 = await dfx.vault.admin_actor.register_policy(request2) as Policy
        to = principalToAddress(Principal.fromText(dfx.vault.id) as any, fromHexString(wallet2.uid))
        vault3 = await dfx.vault.actor_member_2.register_vault(request) as Vault
        wallet3 = await dfx.vault.actor_member_2.register_wallet({name: ["Wallet2"], vault_id: 3n}) as Wallet
    });

    after(() => {
        DFX.STOP();
    });

    it("Transaction register required 1 approves", async function () {
        await setMemberThreshold([1])

        let expectedTransaction: Transaction = getDefaultTransaction()
        expectedTransaction.block_index = [2n]
        expectedTransaction.state = {'Approved': null}

        let actualTransaction = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: tokens,
            wallet_id: wallet1.uid
        }) as Transaction

        expect(actualTransaction.id).eq(1n)

        verifyTransaction(actualTransaction, expectedTransaction);

        let transactions = await dfx.vault.admin_actor.get_transactions() as Array<Transaction>

        expect(transactions.length).eq(1)
        let transactionsMember = await dfx.vault.actor_member_1.get_transactions() as Array<Transaction>
        expect(transactionsMember.length).eq(1)
        expect(transactionsMember[0].id).eq(transactions[0].id)
    });

    it("Transaction required 2 approves", async function () {
        await setMemberThreshold([2])
        let expectedTransaction: Transaction = getDefaultTransaction()

        let actualTransaction = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: tokens,
            wallet_id: wallet1.uid
        }) as Transaction

        verifyTransaction(actualTransaction, expectedTransaction);
        expect(actualTransaction.id).eq(2n)

        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress1,
            status: {'Approved': null}
        })
        expectedTransaction.block_index[0] = 3n
        expectedTransaction.state = {'Approved': null}

        let completed = await dfx.vault.actor_member_1.approve_transaction({
            state: {'Approved': null},
            transaction_id: actualTransaction.id
        }) as Transaction

        verifyTransaction(completed, expectedTransaction)

        expect(completed.modified_date > actualTransaction.modified_date).eq(true)
        expect(completed.created_date === actualTransaction.created_date).eq(true)

        let transactions = await dfx.vault.admin_actor.get_transactions() as Array<Transaction>
        expect(transactions.length).eq(2)
    });

    it("Transaction required 3 approves", async function () {
        await setMemberThreshold([3])

        let expectedTransaction: Transaction = getDefaultTransaction()
        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress1,
            status: {'Approved': null}
        })

        let tr = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: tokens,
            wallet_id: wallet1.uid
        }) as Transaction

        let actualTransaction = await dfx.vault.actor_member_1.approve_transaction({
            state: {'Approved': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(actualTransaction, expectedTransaction);
        expect(actualTransaction.modified_date > tr.modified_date).eq(true);
        expect(tr.created_date === actualTransaction.created_date).eq(true);

        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress2,
            status: {'Approved': null}
        })
        expectedTransaction.block_index[0] = 4n
        expectedTransaction.state = {'Approved': null}

        let completed = await dfx.vault.actor_member_2.approve_transaction({
            state: {'Approved': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(completed, expectedTransaction);
    });

    it("Transaction rejected on 3 approve", async function () {
        await setMemberThreshold([3])

        let expectedTransaction: Transaction = getDefaultTransaction()
        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress1,
            status: {'Approved': null}
        })

        let tr = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: tokens,
            wallet_id: wallet1.uid
        }) as Transaction

        let actualTransaction = await dfx.vault.actor_member_1.approve_transaction({
            state: {'Approved': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(actualTransaction, expectedTransaction);
        expect(actualTransaction.modified_date > tr.modified_date).eq(true);
        expect(tr.created_date === actualTransaction.created_date).eq(true);

        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress2,
            status: {'Rejected': null}
        })
        expectedTransaction.block_index = []
        expectedTransaction.state = {'Rejected': null}

        let completed = await dfx.vault.actor_member_2.approve_transaction({
            state: {'Rejected': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(completed, expectedTransaction);
    });

    it("Transaction canceled on 3 approve", async function () {
        await setMemberThreshold([3])

        let expectedTransaction: Transaction = getDefaultTransaction()
        expectedTransaction.approves.push({
            created_date: 0n,
            signer: memberAddress1,
            status: {'Approved': null}
        })

        let tr = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: tokens,
            wallet_id: wallet1.uid
        }) as Transaction

        let actualTransaction = await dfx.vault.actor_member_1.approve_transaction({
            state: {'Approved': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(actualTransaction, expectedTransaction);
        expect(actualTransaction.modified_date > tr.modified_date).eq(true);
        expect(tr.created_date === actualTransaction.created_date).eq(true);
        expectedTransaction.approves.shift()
        expectedTransaction.approves.push({
            created_date: 0n,
            signer: adminAddress,
            status: {'Canceled': null}
        })
        expectedTransaction.block_index = []
        expectedTransaction.state = {'Canceled': null}

        let completed = await dfx.vault.admin_actor.approve_transaction({
            state: {'Canceled': null},
            transaction_id: tr.id
        }) as Transaction

        verifyTransaction(completed, expectedTransaction);
    });

    it("Transaction insufficient funds", async function () {
        await setMemberThreshold([1])
        let expectedTransaction: Transaction = getDefaultTransaction()
        expectedTransaction.block_index = []
        expectedTransaction.state = {'Rejected': null}
        expectedTransaction.from = wallet2.uid
        expectedTransaction.amount = 10000000000n
        let actualTransaction = await dfx.vault.admin_actor.register_transaction({
            address: to,
            amount: 10000000000n,
            wallet_id: wallet2.uid
        }) as Transaction
        let memo: string = actualTransaction.memo[0]
        expect(memo)
            .contains("ledger transfer error: InsufficientFunds { balance: Tokens { e8s: 300000 } }")
        expectedTransaction.memo = actualTransaction.memo
        verifyTransaction(actualTransaction, expectedTransaction);
    });


    it("Negative scenarios", async function () {
        try {
            await dfx.vault.admin_actor.approve_transaction({
                state: {'Approved': null},
                transaction_id: 1n
            })
        } catch (e) {
            expect(e.message).contains("Transaction not pending")
        }
        try {
            await dfx.vault.admin_actor.approve_transaction({
                state: {'Canceled': null},
                transaction_id: 1n
            })
        } catch (e) {
            expect(e.message).contains("Transaction not pending")
        }
        try {
            await dfx.vault.admin_actor.approve_transaction({
                state: {'Pending': null},
                transaction_id: 1n
            })
        } catch (e) {
            expect(e.message).contains("Transaction not pending")
        }
        try {
            await dfx.vault.admin_actor.approve_transaction({
                state: {'Rejected': null},
                transaction_id: 1n
            })
        } catch (e) {
            expect(e.message).contains("Transaction not pending")
        }
        try {
            await dfx.vault.admin_actor.approve_transaction({
                state: {'Approved': null},
                transaction_id: 100n
            })
        } catch (e) {
            expect(e.message).contains("Nonexistent key error")
        }
        try {
            //wallet registered with another actor (current not member)
            await dfx.vault.admin_actor.register_transaction({
                address: to,
                amount: tokens,
                wallet_id: wallet3.uid
            })
        } catch (e) {
            expect(e.message).contains("Unauthorised")
        }
    });

    it("Check upgrade", async function () {
        DFX.UPGRADE_FORCE('vault')
        let policies = await dfx.vault.admin_actor.get_policies(1n) as [Policy]
        expect(policies.length > 0).eq(true)
        let vaults = await dfx.vault.admin_actor.get_vaults() as [Vault]
        expect(vaults.length > 0).eq(true)
        let wallets = await dfx.vault.admin_actor.get_wallets(1n) as [Wallet]
        expect(wallets.length > 0).eq(true)
        let transactions = await dfx.vault.admin_actor.get_transactions() as Array<Transaction>
        expect(transactions.length > 0).eq(true)
    });

    it("Get backup", async function () {
        try {
            await dfx.vault.admin_actor.get_all_json(0, 10, {'Vaults': null})
            fail("Should unauthorised")
        } catch (e) {
            expect(e.message).contains("Unauthorised")
            DFX.USE_TEST_ADMIN();
            DFX.ADD_CONTROLLER(dfx.user.identity.getPrincipal().toText(), "vault");
            DFX.ADD_CONTROLLER(dfx.vault.id, "vault");
        }
        await dfx.vault.admin_actor.sync_controllers()
        let cVaults = await dfx.vault.admin_actor.count({'Vaults': null}) as number
        let vaultsString = await dfx.vault.admin_actor.get_all_json(0, 10, {'Vaults': null}) as string
        let vaults = JSON.parse(vaultsString) as [Vault]
        expect(vaults.length).eq(3)
        expect(cVaults).eq(3n)
        let walletString = await dfx.vault.admin_actor.get_all_json(0, 10, {'Wallets': null}) as string
        let cWallets = await dfx.vault.admin_actor.count({'Wallets': null}) as number
        let wallets = JSON.parse(walletString) as [Wallet]
        expect(wallets.length).eq(3)
        expect(cWallets).eq(3n)
        let cTr = await dfx.vault.admin_actor.count({'Transactions': null}) as number
        let transactionsString = await dfx.vault.admin_actor.get_all_json(0, 10, {'Transactions': null}) as string
        let transactions = JSON.parse(transactionsString) as [Transaction]
        expect(transactions.length).eq(6)
        expect(cTr).eq(6n)
        let cPolicy = await dfx.vault.admin_actor.count({'Policies': null}) as number
        let policyString = await dfx.vault.admin_actor.get_all_json(0, 10, {'Policies': null}) as string
        let policies = JSON.parse(policyString) as [Policy]
        expect(policies.length).eq(5)
        expect(cPolicy).eq(5n)
        let usersString = await dfx.vault.admin_actor.get_all_json(0, 10, {'Users': null}) as string
        let cUsers = await dfx.vault.admin_actor.count({'Users': null}) as number
        expect(cUsers).eq(3n)
        expect(usersString).contains(adminAddress)
        expect(usersString).contains(memberAddress1)
        expect(usersString).contains(memberAddress2)
    });


    function verifyTransaction(actual: Transaction, expected: Transaction) {
        expect(actual.to).eq(expected.to);
        expect(actual.member_threshold).eq(expected.member_threshold);
        expect(actual.block_index.length).eq(expected.block_index.length);
        if (expected.block_index.length > 0) {
            expect(actual.block_index[0]).eq(expected.block_index[0]);
        }
        expect(actual.owner).eq(expected.owner);
        expect(actual.from).eq(expected.from);
        expect(actual.modified_date >= actual.created_date).eq(true);
        expect(actual.memo.length).eq(expected.memo.length);
        if (expected.memo.length > 0) {
            expect(actual.memo[0]).eq(expected.memo[0]);
        }
        expect(actual.vault_id).eq(expected.vault_id);
        expect(actual.amount_threshold).eq(expected.amount_threshold);
        expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
        expect(actual.approves.length).eq(expected.approves.length);
        for (const expectedApproves of expected.approves) {
            let actualApprove = actual.approves.find(l => l.signer === expectedApproves.signer)
            expect(Object.keys(actualApprove.status)[0]).eq(Object.keys(expectedApproves.status)[0])
            expect(actualApprove.created_date !== 0n).true
        }
        expect(Object.keys(actual.currency)[0])
            .eq(Object.keys(expected.currency)[0]);
        expect(actual.amount).eq(expected.amount);
        expect(actual.created_date > 0).eq(true);
        expect(actual.policy_id).eq(expected.policy_id);
    }

    function getDefaultTransaction(): Transaction {
        let adminApprove: Approve = {
            created_date: 0n,
            signer: adminAddress,
            status: {'Approved': null}
        }
        return {
            amount: 100000n,
            amount_threshold: policy.policy_type.threshold_policy.amount_threshold,
            approves: [adminApprove],
            block_index: [],
            created_date: 0n,
            currency: {'ICP': null},
            from: wallet1.uid,
            id: 0n,
            member_threshold: policy.policy_type.threshold_policy.member_threshold[0],
            memo: [],
            modified_date: 0n,
            owner: adminAddress,
            policy_id: policy.id,
            state: {'Pending': null},
            to: to,
            vault_id: 1n
        }
    }

    async function setMemberThreshold(i: [number] | []) {
        policy.policy_type.threshold_policy.member_threshold = i
        policy = await dfx.vault.admin_actor.update_policy(policy) as Policy
    }

});