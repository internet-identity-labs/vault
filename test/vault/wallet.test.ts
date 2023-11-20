// import "mocha";
// import {deploy} from "../util/deployment.util";
// import {Dfx} from "../type/dfx";
// import {App} from "../constanst/app.enum";
// import {Wallet} from "../idl/vault";
// import {expect} from "chai";
// import {principalToAddress} from "ictool"
// import {DFX} from "../constanst/dfx.const";
//
// let memberAddress: string;
// describe("Wallet", () => {
//     let dfx: Dfx;
//
//     before(async () => {
//         dfx = await deploy({apps: [App.Vault]});
//         memberAddress = principalToAddress(
//             dfx.vault.member_1.getPrincipal() as any);
//
//         await dfx.vault.admin_actor.register_vault({
//             description: [],
//             name: "vault1"
//         })
//         await dfx.vault.admin_actor.register_vault({
//             description: [],
//             name: "vault2"
//         })
//         await dfx.vault.admin_actor.store_member({
//             address: memberAddress,
//             name: ["MoyaLaskovayaSuchechka"],
//             role: {'Member': null},
//             vault_id: 1n,
//             state: {'Active': null},
//         });
//     });
//
//     after(() => {
//         DFX.STOP();
//     });
//
//     let wallet1: Wallet;
//     let wallet2: Wallet;
//
//     it("wallet register", async function () {
//
//         wallet1 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet1"], vault_id: 1n}) as Wallet
//         verifyWallet(wallet1, {
//             created_date: 0n,
//             uid: wallet1.uid,
//             modified_date: 0n,
//             name: ["Wallet1"],
//             state: {'Active': null},
//             vaults: [1n]
//
//         })
//         wallet2 = await dfx.vault.admin_actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
//         verifyWallet(wallet2, {
//             created_date: 0n,
//             uid: wallet2.uid,
//             modified_date: 0n,
//             name: ["Wallet2"],
//             state: {'Active': null},
//             vaults: [1n]
//
//         })
//         let wallets = await dfx.vault.admin_actor.get_wallets(1n) as [Wallet]
//         expect(wallets.length).eq(2)
//         let wallet1_1 = wallets.find(l => l.uid === wallet1.uid)
//         let wallet2_1 = wallets.find(l => l.uid === wallet2.uid)
//         verifyWallet(wallet1_1, {
//             created_date: wallet1.created_date,
//             uid: wallet1.uid,
//             modified_date: 0n,
//             name: ["Wallet1"],
//             state: {'Active': null},
//             vaults: [1n]
//
//         })
//         verifyWallet(wallet2_1, {
//             created_date: wallet2.created_date,
//             uid: wallet2.uid,
//             modified_date: 0n,
//             name: ["Wallet2"],
//             state: {'Active': null},
//             vaults: [1n]
//         })
//     });
//     it("update wallet", async function () {
//         let updated = await dfx.vault.admin_actor.update_wallet({
//             created_date: 321n,
//             uid: wallet1.uid,
//             modified_date: 123n,
//             name: ["Wallet1_Udated"],
//             state: {'Archived': null},
//             vaults: [2n]
//
//         }) as Wallet
//         wallet1.name = ["Wallet1_Udated"]
//         wallet1.state = {'Archived': null}
//         verifyWallet(wallet1, updated)
//         expect(wallet1.modified_date !== updated.modified_date).true
//     })
//
//     it("update wallet negative", async function () {
//         try {
//             await dfx.vault.actor_member_1.update_wallet({
//                 created_date: 321n,
//                 uid: wallet1.uid,
//                 modified_date: 123n,
//                 name: ["Wallet1_Udated"],
//                 state: {'Archived': null},
//                 vaults: [2n]
//
//             })
//         } catch (e: any) {
//             expect(e.message.includes("Not enough permissions")).eq(true)
//         }
//     })
//
//     it("register wallet negative ", async function () {
//         try {
//             await dfx.vault.actor_member_1.get_wallets(2n)
//         } catch (e: any) {
//             expect(e.message.includes("Unauthorised")).eq(true)
//         }
//         try {
//             await dfx.vault.actor_member_1.get_wallets(3n)
//         } catch (e: any) {
//             expect(e.message.includes("Nonexistent key")).eq(true)
//         }
//         try {
//             await dfx.vault.actor_member_1.register_wallet({name: ["Wallet1"], vault_id: 3n})
//         } catch (e: any) {
//             expect(e.message.includes("Nonexistent key")).eq(true)
//         }
//         try {
//             await dfx.vault.actor_member_1.register_wallet({name: ["Wallet1"], vault_id: 2n})
//         } catch (e: any) {
//             expect(e.message.includes("Unauthorised")).eq(true)
//         }
//         try {
//             await dfx.vault.actor_member_1.register_wallet({name: ["Wallet1"], vault_id: 1n})
//         } catch (e: any) {
//             expect(e.message.includes("Not enough permissions")).eq(true)
//         }
//     });
// });
//
//
// function verifyWallet(actual: Wallet, expected: Wallet) {
//     expect(actual.vaults.length).eq(expected.vaults.length)
//     if (actual.vaults.length > 0) {
//         expect(actual.vaults[0]).eq(expected.vaults[0])
//     }
//     expect(actual.name.length).eq(expected.name.length)
//     if (actual.name.length > 0) {
//         expect(actual.name[0]).eq(expected.name[0])
//     }
//     expect(actual.uid).eq(expected.uid)
//     expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
// }