import {idlFactory} from "../../test/vault_repo/sdk/vr_idl";
import {VaultWasm, VersionWrapper} from "../../test/vault_repo/sdk/vr";
import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";
import {Ed25519KeyIdentity} from "@dfinity/identity";

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
async function synchroniseWASMRepos() {
    const wasmFilePath = process.env.WASM_FILE_PATH;
    const version = process.env.VERSION;
    const canisterIdProd = process.env.CANISTER_ID; //TODO prod canister id
    const canisterId = process.env.CANISTER_ID; //TODO target canister id
    const identitySeed = process.env.IDENTITY_SEED; //TODO only dev/stage
    const description = process.env.DESCRIPTION;

    if (!wasmFilePath || !version || !canisterId || !identitySeed) {
        throw new Error("Please provide WASM_FILE_PATH, VERSION, CANISTER_ID, IDENTITY_SEED as environment variables.");
    }

    console.log({wasmFilePath,version,canisterId,description});

    //clean vault_repo
    //before this run `dfx canister install vault_repo '(ARGUMENTS DEPENDS ON NETWORK)' --network DEV/STAGE  `

    try {
        const identity = getIdentity(identitySeed);
        const prodActor = await getActor(canisterIdProd, identity, idlFactory);
        const allVersions = await prodActor.get_available_versions(version) as Array<VersionWrapper>;
        const actor = await getActor(canisterId, identity, idlFactory);
       for (const versionWrapper of allVersions) {
           let wasm = await prodActor.get_by_version(versionWrapper.version) as VaultWasm;
            await actor.add_version(wasm);
       }
    } catch (error) {
        throw new Error(`Error while saving Wasm file: ${error}`);
    }
}

synchroniseWASMRepos();
