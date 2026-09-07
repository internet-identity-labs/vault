//! Canister creation cost as reported by the replica.
//!
//! The cost is not a constant: the system charges it proportionally to the size of the
//! subnet, so the same call is roughly 2.6 times more expensive on the 34 node fiduciary
//! subnet than on a regular 13 node one. Instead of hardcoding a number that silently
//! becomes wrong the moment the manager moves, we ask the replica through the
//! `ic0.cost_create_canister` system API.

#[cfg(target_arch = "wasm32")]
mod ic0 {
    #[link(wasm_import_module = "ic0")]
    extern "C" {
        /// Writes the 128 bit cost, little endian, into 16 bytes at `dst`.
        pub fn cost_create_canister(dst: usize);
    }
}

/// Cycles the system currently charges for `create_canister` on this subnet.
///
/// Returns an error rather than a fallback value: funding a vault with a guessed
/// cost either underfunds the vault or overcharges the user, and both are worse
/// than refusing the call.
#[cfg(target_arch = "wasm32")]
pub fn create_canister_cost() -> Result<u128, String> {
    let mut bytes = [0u8; 16];
    unsafe {
        ic0::cost_create_canister(bytes.as_mut_ptr() as usize);
    }
    let cost = u128::from_le_bytes(bytes);
    if cost == 0 {
        // Creating a canister is never free, so a zero means the replica did not
        // answer and we must not proceed with a made up number.
        return Err("The replica reported a zero canister creation cost".to_string());
    }
    Ok(cost)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_canister_cost() -> Result<u128, String> {
    Err("The canister creation cost is only available on the replica".to_string())
}
