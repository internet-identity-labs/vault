import { idlFactory } from "../../test/vault_repo/sdk/vr_idl";
import { VaultWasm } from "../../test/vault_repo/sdk/vr";

import { sha256 } from "ethers/lib/utils";
import * as fs from "fs";
import { Actor, ActorMethod, HttpAgent, Identity } from "@dfinity/agent";
import { IDL } from "@dfinity/candid";
import { Ed25519KeyIdentity } from "@dfinity/identity";

/**
 * Read Wasm file and return as Uint8Array.
 * @param {string} filePath - Path to the Wasm file.
 * @returns {Uint8Array} - Wasm file content.
 */
export function readWasmFile(filePath: string): Uint8Array {
    try {
        const buffer = fs.readFileSync(filePath);
        return new Uint8Array(buffer);
    } catch (error) {
        throw new Error(`Error reading Wasm file from ${filePath}: ${error}`);
    }
}

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

/**
 * Save Vault Wasm by adding a new version to the actor.
 * WASM_FILE_PATH  - Path to the WASM file (artifact)
 * VERSION         - Version of the artifact
 * DESCRIPTION     - Description to the version of the artifact
 * CANISTER_ID     - (id of vault_repo canister)
 * IDENTITY_SEED   -
 */
async function saveVaultWasm() {
    const wasmFilePath = process.env.WASM_FILE_PATH;
    const version = process.env.VERSION;
    const canisterId = process.env.CANISTER_ID;
    const identitySeed = process.env.IDENTITY_SEED;
    const description = process.env.DESCRIPTION;

    if (!wasmFilePath || !version || !canisterId || !identitySeed) {
        throw new Error("Please provide WASM_FILE_PATH, VERSION, CANISTER_ID, IDENTITY_SEED as environment variables.");
    }

    console.log({wasmFilePath,version,canisterId,description});
    
    try {
        const wasmBytes = readWasmFile(wasmFilePath);
        const identity = getIdentity(identitySeed);
        const principal = identity.getPrincipal().toText();
        const actor = await getActor(canisterId, identity, idlFactory);
        const hash = sha256(wasmBytes);
        const wasm: VaultWasm = {
            wasm_module: Array.from(wasmBytes),
            hash: hash,
            version: version,
            description: description ? [description] : [],
        };

        console.log({principal});

        await actor.add_version(wasm);
        console.log("Version was sent.");

        // Check if the version is present in the actor
        const retrievedWasm = await actor.get_by_version(version) as VaultWasm;
        console.log({retrievedWasmHash:retrievedWasm.hash, hash, retrievedWasmVersion:retrievedWasm.version, version, retrievedWasmDescription:retrievedWasm.description, description});

        if (retrievedWasm.hash === hash && retrievedWasm.version === version) {
            console.log("Wasm file saved successfully.");
        } else {
            throw new Error("Data not match expectation");
        }
    } catch (error) {
        throw new Error(`Error while saving Wasm file: ${error}`);
    }
}

saveVaultWasm();
