import { idlFactory } from "../../test/vault_manager/sdk/vm_idl";
import { idlFactory as VaultIdl } from "./idl/idl";

import * as fs from "fs";
import {Actor, ActorMethod, HttpAgent, Identity, SignIdentity} from "@dfinity/agent";
import { IDL } from "@dfinity/candid";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import {VaultCanister} from "../../test/vault_manager/sdk/vm";

require('./../../test/vault/bigintextension.js');


/**
 * Get an actor instance for the specified canister ID and identity.
 * @param {string} canisterId - Canister ID.
 * @param {Identity} identity - Identity for authentication.
 * @param {IDL.InterfaceFactory} idl - Interface factory for the canister.
 * @returns {Promise<Record<string, ActorMethod>>} - Actor instance.
 */
export const getActor = async (
    canisterId: string,
    identity: Identity,
    idl: IDL.InterfaceFactory
): Promise<Record<string, ActorMethod>> => {
    const agent: HttpAgent = new HttpAgent({ host: "https://ic0.app/", identity: identity });
    // await agent.fetchRootKey();
    return Actor.createActor(idl, { agent, canisterId });
};

/**
 * Get an Ed25519KeyIdentity based on the provided seed.
 * @param {string} seed - Seed for identity generation.
 * @returns {Ed25519KeyIdentity} - Identity instance.
 */
export const getIdentity = (seed: string): Ed25519KeyIdentity => {
    const seedEncoded = new TextEncoder().encode(seed);
    return Ed25519KeyIdentity.generate(seedEncoded);
};

export async function getCanisters(canister_id: string, identity: SignIdentity): Promise<[VaultCanister]> {
    let actor = await getActor(canister_id, identity, idlFactory);
    return await actor.get_all_canisters() as [VaultCanister];
}

/**
 * Save provided data to file with provided path.
 * @param {string} filePath - Path to the file which need to be written.
 * @param {string} dataToWrite - Data which need to be saved.
 */
async function writeToFile(filePath, dataToWrite) {
    if (!filePath) {
        throw new Error("File path was not provided.");
    }
    console.log(`Working with '${filePath}'`);

    await fs.access(filePath, fs.constants.F_OK, (err) => {
        if (!err) {
            throw new Error(`File '${filePath}' already exists.`);
        }
      
        fs.writeFile(filePath, dataToWrite, (writeErr) => {
            if (writeErr) {
                throw new Error(`Error during record to file '${filePath}'`);
            }
            console.log(`Write done to '${filePath}' \n`);
        });
    });
}

/**
 * Function will create backup of the vault_manager and data
 * CANISTER_ID    - (id of vault_manager canister)
 * IDENTITY_SEED  - Seed for the Identity
 * BACKUP_PATH    - Path to the backup folder
 */
async function createBackupAll() {
    const canisterId = process.env.CANISTER_ID;
    const identitySeed = process.env.IDENTITY_SEED;
    const backupPath = process.env.BACKUP_PATH;

    const identity = getIdentity(identitySeed);

    if (!canisterId || !identitySeed || !backupPath) {
        throw new Error("Please provide CANISTER_ID, IDENTITY_SEED, BACKUP_PATH as environment variables.");
    }

    let canisters: [VaultCanister] = await getCanisters(canisterId, identity);
    canisters.sort();
    let response = JSON.stringify(canisters);

    let vault_manager_path = `${backupPath}/vault_manager_${canisterId}.json`;
    await writeToFile(vault_manager_path, response);

    let canisterIds = canisters
        .map((canister) => canister.canister_id)
        .map((canister_id) => canister_id.toText());

    console.log("All canisters id.");
    canisterIds.sort();
    console.log(canisterIds);

    for (let i = 0; i < canisterIds.length; i++) {
        let canister_path = `${backupPath}/${canisterIds[i]}.json`
        let vaultActor = await getActor(canisterIds[i], identity, VaultIdl);
        let transactions = await vaultActor.get_transactions_all();
        let transactionsResponse = JSON.stringify(transactions);

        await writeToFile(canister_path, transactionsResponse);
    }
}


createBackupAll();
