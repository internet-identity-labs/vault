import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {Vault, VaultMember,} from "../idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "../constanst/dfx.const";

let rootAddress: string;
let memberAddress: string;

describe("Vault", () => {
    let dfx: Dfx;


    before(async () => {
        dfx = await deploy({apps: [App.Vault]});

        rootAddress = principalToAddress(
            dfx.user.identity.getPrincipal() as any,
            Array(32).fill(1));

        memberAddress = principalToAddress(
            dfx.vault.member_1.getPrincipal() as any,
            Array(32).fill(1));
    });

    after(() => {
        DFX.STOP();
    });

    it("get_vaults empty", async function () {
        let vaults = await dfx.vault.admin_actor.get_vaults() as [Vault]
        expect(vaults.length).eq(0)
    });
    let vault: Vault;

    it("register_vault", async function () {

        vault = await dfx.vault.admin_actor.register_vault({
            description: [],
            name: "vault1"
        }) as Vault
        let expectedVault = getExpectedVault()

        verifyVault(vault, expectedVault)
        expect(vault.modified_date === vault.created_date).true

        let vaultRegisterRequest2 = {
            description: ["test2"],
            name: "vault2"
        };
        let vault2 = await dfx.vault.admin_actor.register_vault(vaultRegisterRequest2) as Vault

        expectedVault.name = "vault2";
        expectedVault.id = 2n;
        expectedVault.description = ["test2"];

        verifyVault(vault2, expectedVault)
    });

    it("get_vaults", async function () {
        let vaults = await dfx.vault.admin_actor.get_vaults() as [Vault]
        let vault: Vault = vaults.find(l => l.id === 1n);
        let vault2: Vault = vaults.find(l => l.id === 2n);
        let expected = getExpectedVault();
        verifyVault(vault, expected)

        expected.name = "vault2";
        expected.id = 2n;
        expected.description = ["test2"];
        verifyVault(vault2, expected)
    });

    it("add_member", async function () {

        let member = await dfx.vault.admin_actor.store_member({
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            state: {'Active': null},
            vault_id: 1n
        }) as Vault;

        let expected = getExpectedVault()
        expected.members.push(getDefaultMember())
        verifyVault(member, expected)
        expect(member.modified_date > member.created_date).true

        let vaultsForMember = (await dfx.vault.actor_member_1.get_vaults()) as [Vault];
        expect(vaultsForMember.length).eq(1)
        let vaultForMember = vaultsForMember[0]
        verifyVault(vaultForMember, expected)

        vault = (await dfx.vault.admin_actor.get_vaults() as [Vault])
            .find(l => l.id === 1n)
        verifyVault(vault, expected)

        let vaultMember = (await dfx.vault.actor_member_1.get_vaults() as [Vault])
            .find(l => l.id === 1n)
        verifyVault(vaultMember, expected)
    });


    it("update vault/member", async function () {
        let vault = (await dfx.vault.actor_member_1.get_vaults() as [Vault])
            .find(l => l.id === 1n)

        let request = structuredClone(vault);
        request.name = "Updated name";
        request.description = ["Updated description"];
        request.state = {'Archived': null};
        request.members.find(l => l.user_uuid === memberAddress).state = {'Archived': null}

        let updated = await dfx.vault.admin_actor.update_vault(request) as Vault;

        let expected = structuredClone(vault);
        expected.name = "Updated name";
        expected.description = ["Updated description"];
        expected.state = {'Archived': null};

        //does not update members
        verifyVault(updated, expected)
        expect(request.modified_date !== updated.modified_date).true

        expected.members.find(l => l.user_uuid === memberAddress).state = {'Archived': null}
        expected.members.find(l => l.user_uuid === memberAddress).name = []

        let vaultMember = (await dfx.vault.actor_member_1.get_vaults() as [Vault])
            .find(l => l.id === 1n)
        let archivedMemberVaylt = await dfx.vault.admin_actor.store_member({
            address: memberAddress,
            name: [],
            role: {'Member': null},
            state: {'Archived': null},
            vault_id: 1n
        }) as Vault;

        expect(vaultMember.modified_date !== archivedMemberVaylt.modified_date).true
        verifyVault(archivedMemberVaylt, expected)
    })


    it("negative scenarios for update vault", async function () {
        try {
            let request = vault;
            request.id = 1n
            await dfx.vault.actor_member_1.update_vault(request);
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
        try {
            let request = vault;
            request.id = 2n
            await dfx.vault.actor_member_1.update_vault(request);
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
    });

    it("negative scenarios for store members", async function () {
        try {
            await dfx.vault.actor_member_1.store_member({
                address: memberAddress,
                name: ["Moya Laskovaya Suchechka"],
                role: {'Member': null},
                vault_id: 2n,
                state: {'Active': null},
            })
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        try {
            await dfx.vault.actor_member_1.store_member({
                address: memberAddress,
                name: ["Moya Laskovaya Suchechka"],
                role: {'Member': null},
                vault_id: 1n,
                state: {'Active': null},
            })
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });
});


function verifyVault(actualVault: Vault, expectedVault: Vault) {
    expect(actualVault.name).eq(expectedVault.name)
    expect(actualVault.id).eq(expectedVault.id)
    expect(Object.keys(actualVault.state)[0]).eq(Object.keys(expectedVault.state)[0])
    expect(actualVault.description.length).eq(expectedVault.description.length)
    if (actualVault.description.length > 0) {
        expect(actualVault.description[0]).eq(expectedVault.description[0])
    }
    expect(actualVault.wallets.length).eq(expectedVault.wallets.length)
    for (const actWallet of actualVault.wallets) {
        expect(actualVault.wallets.includes(actWallet))
    }
    expect(actualVault.policies.length).eq(expectedVault.policies.length)
    for (const actPolicy of actualVault.policies) {
        expect(actualVault.policies.includes(actPolicy))
    }
    expect(actualVault.modified_date !== 0n).true
    expect(actualVault.members.length).eq(expectedVault.members.length)
    for (const actMember of actualVault.members) {
        verifyMember(actMember, expectedVault.members.find(l => l.user_uuid === actMember.user_uuid))
    }

}

function verifyMember(actual: VaultMember, expected: VaultMember) {
    expect(actual.name.length).eq(expected.name.length)
    if (actual.name.length > 0) {
        expect(actual.name[0]).eq(expected.name[0])
    }
    expect(actual.user_uuid).eq(expected.user_uuid)
    expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
    expect(Object.keys(actual.role)[0]).eq(Object.keys(expected.role)[0])
}

function getExpectedVault(): Vault {
    return {
        created_date: 0n,
        description: [],
        id: 1n,
        members: [
            {
                name: [],
                role: {'Admin': null},
                state: {'Active': null},
                user_uuid: rootAddress
            }
        ],
        modified_date: 0n,
        name: 'vault1',
        policies: [1n],
        state: {'Active': null},
        wallets: []
    }
}

function getDefaultMember(): VaultMember {
    return {
        state: {'Active': null},
        user_uuid: memberAddress,
        name: ["MoyaLaskovayaSuchechka"],
        role: {'Member': null},
    }
}