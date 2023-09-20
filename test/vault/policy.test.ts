import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {Policy, PolicyRegisterRequest, Transaction, TransactionRegisterRequest, Vault, Wallet} from "../idl/vault";
import {expect} from "chai";
import {fromHexString, principalToAddress} from "ictool"
import {DFX} from "../constanst/dfx.const";
import {Principal} from "@dfinity/principal";


let memberAddress: string;

describe("Policy", async function () {
    var dfx: Dfx;
    let wallet1: Wallet;
    let wallet2: Wallet;
    this.timeout(90000);

    before(async () => {
        dfx = await deploy({apps: [App.Vault]});
        memberAddress = principalToAddress(
            dfx.vault.member_1.getPrincipal() as any);
        await dfx.vault.admin_actor.register_vault({
            description: [],
            name: "vault1"
        })
        await dfx.vault.admin_actor.register_vault({
            description: [],
            name: "vault2"
        })
        await dfx.vault.admin_actor.store_member({
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n,
            state: {'Active': null},
        });
        wallet1 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet1"], vault_id: 2n}) as Wallet
        wallet2 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
    });

    after(() => {
        DFX.STOP();
    });

    let defaultPolicy1: Policy;
    let defaultPolicy2: Policy;
    it("verify default policy", async function () {
        let policies = await dfx.vault.admin_actor.get_policies(1n) as [Policy]
        defaultPolicy1 = policies[0]
        let policies2 = await dfx.vault.admin_actor.get_policies(2n) as [Policy]
        defaultPolicy2 = policies2[0]
        verifyPolicy(policies[0], {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 1n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 0n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: []
                },

            }
        })
    })
    let policy1: Policy;
    let policy2: Policy;
    it("register policy", async function () {

        policy1 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [2],
                    wallets: [[wallet2.uid]]
                }
            },
            state: {'Active': null},
            vault_id: 1n
        } as PolicyRegisterRequest) as Policy
        verifyPolicy(policy1, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 3n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [2],
                    wallets: [[wallet2.uid]]
                },

            }
        })
        expect(policy1.created_date).eq(policy1.modified_date);
        policy2 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: []
                }
            },
            vault_id: 1n
        }) as Policy
        verifyPolicy(policy2, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 4n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: []
                },
            }
        });
        let policies = await dfx.vault.admin_actor.get_policies(1n) as [Policy]
        expect(policies.length).eq(3)
        let policy1_1 = policies.find(l => l.id === 3n)
        let policy2_1 = policies.find(l => l.id === 4n)
        verifyPolicy(policy1_1, policy1)
        verifyPolicy(policy2_1, policy2)
    });

    it("update policy", async function () {
        let policies = await dfx.vault.admin_actor.get_policies(1n) as [Policy]
        let policy = policies.find(l => l.id === 1n)

        let updatePolicyRequest: Policy = {
            created_date: 1n,
            id: 1n,
            modified_date: 2n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: [[wallet2.uid]]
                },

            },
            state: {'Archived': null},
            vault: 66n
        }
        let result = await dfx.vault.admin_actor.update_policy(updatePolicyRequest) as Policy
        policy.state = {'Archived': null};
        policy.policy_type = {
            'threshold_policy': {
                amount_threshold: 2n,
                currency: {'ICP': null},
                member_threshold: [3],
                wallets: [[wallet2.uid]]
            }
        };
        verifyPolicy(result, policy);
        expect(policy.modified_date !== result.modified_date).true

        try {
            await dfx.vault.actor_member_1.update_policy(updatePolicyRequest)
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
        updatePolicyRequest.policy_type.threshold_policy.wallets = [["SOME_FAKE_UID"]]
        try {
            await dfx.vault.admin_actor.update_policy(updatePolicyRequest)
        } catch (e: any) {
            expect(e.message.includes("Stop it!!! Not your wallet!!!")).eq(true)
        }
    })

    it("register policy negative ", async function () {
        try {
            await dfx.vault.actor_member_1.get_policies(2n)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).true
        }
        try {
            await dfx.vault.actor_member_1.get_policies(3n)
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key error")).true
        }
        try {
            await dfx.vault.actor_member_1.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: [2],
                        wallets: []
                    }
                }, vault_id: 3n
            })
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).true
        }
        try {
            await dfx.vault.actor_member_1.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: [2],
                        wallets: []
                    }
                }, vault_id: 2n
            })
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).true
        }
        try {
            await dfx.vault.actor_member_1.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: [2],
                        wallets: []
                    }
                }, vault_id: 1n
            })
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
        try {
            await dfx.vault.admin_actor.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: [2],
                        wallets: [["FAKE_WALLET_ID"]]
                    }
                }, vault_id: 1n
            })
        } catch (e: any) {
            expect(e.message.includes("Stop it!!! Not your wallet!!!")).eq(true)
        }
    });


    let policy3;
    let to;

    it("define correct policy both less than", async function () {
        policy3 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [1],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        to = principalToAddress(Principal.fromText(dfx.vault.id) as any, fromHexString(wallet1.uid))
        let tokens = 100n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy3.id)
    });

    it("define correct policy one greater and one policy less", async function () {
        let tokens = 8n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(defaultPolicy2.id)
    });

    let policy4;

    it("define policy with stronger member threshold", async function () {
        policy4 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [5],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 11n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy4.id)
    });

    let policy5;

    it("define policy with all member threshold when amount same and both wallets all", async function () {
        policy5 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 11n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy5.id)
    });

    let policy6;

    it("define policy with list of wallet uid", async function () {
        policy6 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 15n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: [[wallet1.uid]]
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 20n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy6.id)
    });

    let policy7;

    it("define policy with same threshold empty wallets choose one with wallet_id in the list", async function () {
        policy7 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 15n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 20n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy6.id)
    });

    let policy8;
    let policy9;

    it("define policy with same threshold empty wallets choose one with All members", async function () {
        policy8 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 25n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        policy9 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 25n,
                    currency: {'ICP': null},
                    member_threshold: [2],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 26n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy8.id)
    });

    let policy10;
    let policy11;

    it("define policy with same threshold empty wallets choose one with All members", async function () {
        policy10 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 30n,
                    currency: {'ICP': null},
                    member_threshold: [5],
                    wallets: [[wallet1.uid]]
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        policy11 = await dfx.vault.admin_actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 30n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: [[wallet1.uid]]
                }
            },
            state: {'Active': null},
            vault_id: 2n
        } as PolicyRegisterRequest) as Policy;
        let tokens = 30n;
        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}
        let actualTransaction = await dfx.vault.admin_actor.register_transaction(registerRequest) as Transaction
        expect(actualTransaction.policy_id).eq(policy11.id)
    });

    it("expect: unable to find policy", async function () {
        let vault = await dfx.vault.admin_actor.register_vault({
            description: [],
            name: "vault3"
        }) as Vault
        let wallet3 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet2"], vault_id: vault.id}) as Wallet
        await dfx.vault.admin_actor.update_policy({
            created_date: 1n,
            id: vault.policies[0],
            modified_date: 2n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 20000000n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: [[wallet3.uid]]
                },

            },
            state: {'Archived': null},
            vault: 3n
        })
        try {
            let registerRequest: TransactionRegisterRequest = {address: to, amount: 100n, wallet_id: wallet3.uid}
            await dfx.vault.admin_actor.register_transaction(registerRequest)
        } catch (e) {
            expect(e.message).contains("Unable to find the policy!")
        }
    });
});


function verifyPolicy(actual: Policy, expected: Policy) {
    expect(actual.id).eq(expected.id)
    expect(actual.vault).eq(expected.vault)
    expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
    expect(actual.policy_type.threshold_policy.member_threshold.length)
        .eq(expected.policy_type.threshold_policy.member_threshold.length)
    expect(actual.policy_type.threshold_policy.amount_threshold)
        .eq(expected.policy_type.threshold_policy.amount_threshold)
    expect(actual.policy_type.threshold_policy.wallets.length)
        .eq(expected.policy_type.threshold_policy.wallets.length)
    expect(Object.keys(actual.policy_type.threshold_policy.currency)[0])
        .eq(Object.keys(expected.policy_type.threshold_policy.currency)[0])
    if (actual.policy_type.threshold_policy.wallets.length > 0
        && actual.policy_type.threshold_policy.wallets[0].length > 0) {
        for (const wallet of expected.policy_type.threshold_policy.wallets as string[]) {
            expect((actual.policy_type.threshold_policy.wallets as string[]).includes(wallet))
        }
    }
    if (actual.policy_type.threshold_policy.member_threshold.length > 0) {
        expect(actual.policy_type.threshold_policy.member_threshold[0])
            .eq(expected.policy_type.threshold_policy.member_threshold[0])
    }
}