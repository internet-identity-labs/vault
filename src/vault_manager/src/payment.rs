use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::call;
use ic_ledger_types::{AccountBalanceArgs, AccountIdentifier, DEFAULT_FEE, DEFAULT_SUBACCOUNT, MAINNET_CYCLES_MINTING_CANISTER_ID,
                      MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens, TransferArgs};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use serde::{Deserialize, Serialize};

use crate::config::CONF;

/// "TPUP" - memo the CMC expects for a top up of an existing canister.
pub const MEMO_TOP_UP_CANISTER: Memo = Memo(1347768404);

/// Share of the user payment kept by the protocol, the rest is converted into cycles.
pub const PROTOCOL_CUT_PERCENT: u64 = 10;

/// The ICP ledger transfer fee, charged on every transfer and on top of the amount
/// pulled by `icrc2_transfer_from`.
pub const LEDGER_FEE_E8S: u64 = DEFAULT_FEE.e8s();

/// Every ICP leaving the manager costs one ledger fee. We do two of them:
/// one to the CMC and one to the protocol wallet.
pub const OUTGOING_TRANSFERS: u64 = 2;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PaymentSplit {
    /// Part of the payment converted into cycles on the manager balance.
    pub to_cycles: u64,
    /// Part of the payment sent to the protocol wallet.
    pub to_protocol: u64,
}

/// Splits the user payment between the cycles top up and the protocol revenue.
/// Both outgoing ledger fees are taken from the cycles part.
pub fn split_payment(price: u64) -> Result<PaymentSplit, String> {
    let to_protocol = price / 100 * PROTOCOL_CUT_PERCENT;
    let fees = DEFAULT_FEE.e8s() * OUTGOING_TRANSFERS;
    let to_cycles = match price.checked_sub(to_protocol + fees) {
        None => return Err(format!("Configured price {} does not cover the protocol cut and the ledger fees", price)),
        Some(x) => x,
    };
    Ok(PaymentSplit { to_cycles, to_protocol })
}

/// Candid `nat` is unbounded, both conversions saturate instead of trapping.
pub fn nat_to_u128(n: &Nat) -> u128 {
    let mut result: u128 = 0;
    for digit in n.0.to_u64_digits().iter().rev() {
        match result.checked_shl(64).and_then(|r| r.checked_add(*digit as u128)) {
            None => return u128::MAX,
            Some(x) => result = x,
        }
    }
    result
}

pub fn nat_to_u64(n: &Nat) -> u64 {
    match n.0.to_u64_digits().first() {
        None => 0,
        Some(x) if n.0.to_u64_digits().len() == 1 => *x,
        Some(_) => u64::MAX,
    }
}

#[derive(CandidType, Deserialize)]
struct IcpXdrConversionRate {
    xdr_permyriad_per_icp: u64,
    timestamp_seconds: u64,
}

#[derive(CandidType, Deserialize)]
struct IcpXdrConversionRateResponse {
    data: IcpXdrConversionRate,
    #[serde(with = "serde_bytes")]
    hash_tree: Vec<u8>,
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
}

/// Cycles the CMC would mint for the given amount of e8s under the given rate.
///
/// 1 XDR is 10^12 cycles and the rate is given in permyriad of XDR per ICP, so
/// `e8s / 10^8 * rate / 10^4 * 10^12` collapses into a plain `e8s * rate`.
pub fn e8s_to_cycles(e8s: u64, xdr_permyriad_per_icp: u64) -> u128 {
    (e8s as u128) * (xdr_permyriad_per_icp as u128)
}

pub async fn get_icp_xdr_rate() -> Result<u64, String> {
    let response: (IcpXdrConversionRateResponse, ) = call(
        MAINNET_CYCLES_MINTING_CANISTER_ID,
        "get_icp_xdr_conversion_rate",
        (),
    ).await.map_err(|(code, msg)| format!("Failed to read the ICP/XDR rate: {}: {}", code as u8, msg))?;
    Ok(response.0.data.xdr_permyriad_per_icp)
}

/// Pulls `amount` e8s from the user default account into the manager default account.
/// Requires an ICRC-2 allowance of at least `amount + DEFAULT_FEE` granted to the manager.
pub async fn charge_user(user: Principal, amount: u64) -> Result<Nat, String> {
    let args = TransferFromArgs {
        spender_subaccount: None,
        from: Account { owner: user, subaccount: None },
        to: Account { owner: ic_cdk::id(), subaccount: None },
        amount: Nat::from(amount),
        fee: Some(Nat::from(DEFAULT_FEE.e8s())),
        memo: None,
        created_at_time: None,
    };

    let response: (Result<Nat, TransferFromError>, ) = call(
        MAINNET_LEDGER_CANISTER_ID,
        "icrc2_transfer_from",
        (args, ),
    ).await.map_err(|(code, msg)| format!("Failed to call the ledger: {}: {}", code as u8, msg))?;

    response.0.map_err(|e| match e {
        TransferFromError::InsufficientAllowance { allowance } =>
            format!("Insufficient allowance: {}. Approve at least {} e8s for the vault manager",
                    allowance, amount + DEFAULT_FEE.e8s()),
        TransferFromError::InsufficientFunds { balance } =>
            format!("Insufficient funds: {}", balance),
        other => format!("Payment failed: {:?}", other),
    })
}

/// Returns `amount` e8s back to the user default account, minus the ledger fee.
pub async fn refund_user(user: Principal, amount: u64) -> Result<u64, String> {
    let refund = match amount.checked_sub(DEFAULT_FEE.e8s()) {
        None => return Err("Nothing left to refund after the ledger fee".to_string()),
        Some(x) => x,
    };
    transfer_from_manager(
        AccountIdentifier::new(&user, &DEFAULT_SUBACCOUNT),
        refund,
        Memo(0),
    ).await
}

/// ICP held on the manager default account.
///
/// Money can end up here without belonging to anyone: a refund that failed to reach
/// the user, a payment whose CMC top up did not go through, or the remainder of a
/// split that was only partly settled.
pub async fn manager_icp_balance() -> Result<u64, String> {
    let account = AccountIdentifier::new(&ic_cdk::id(), &DEFAULT_SUBACCOUNT);
    ic_ledger_types::account_balance(MAINNET_LEDGER_CANISTER_ID, AccountBalanceArgs { account })
        .await
        .map(|tokens| tokens.e8s())
        .map_err(|(code, msg)| format!("Failed to read the manager balance: {}: {}", code as u8, msg))
}

/// Moves ICP off the manager default account to an arbitrary account.
pub async fn sweep_to(to: AccountIdentifier, amount: u64) -> Result<u64, String> {
    transfer_from_manager(to, amount, Memo(0)).await
}

/// Parses a destination for a sweep: either a hex account identifier or a principal,
/// whose default account is used.
pub fn parse_destination(destination: &str) -> Result<AccountIdentifier, String> {
    if let Ok(account) = AccountIdentifier::from_hex(destination) {
        return Ok(account);
    }
    match Principal::from_text(destination) {
        Ok(owner) => Ok(AccountIdentifier::new(&owner, &DEFAULT_SUBACCOUNT)),
        Err(e) => Err(format!(
            "{} is neither an account identifier nor a principal: {}",
            destination, e
        )),
    }
}

/// Sends the protocol share to the wallet configured as `destination_address`.
pub async fn pay_protocol(amount: u64) -> Result<u64, String> {
    let destination = CONF.with(|c| c.borrow().destination_address.clone());
    let to = AccountIdentifier::from_hex(&destination)
        .map_err(|e| format!("Invalid destination address {}: {}", destination, e))?;
    transfer_from_manager(to, amount, Memo(0)).await
}

/// Converts `amount` e8s into cycles on the manager own balance and returns the
/// amount of cycles minted by the CMC.
pub async fn top_up_self(amount: u64) -> Result<u128, String> {
    let manager = ic_cdk::id();
    let cmc_account = AccountIdentifier::new(
        &MAINNET_CYCLES_MINTING_CANISTER_ID,
        &Subaccount::from(manager),
    );

    let block_index = transfer_from_manager(cmc_account, amount, MEMO_TOP_UP_CANISTER).await?;
    notify_top_up(block_index, manager).await
}

#[derive(CandidType, Deserialize)]
struct NotifyTopUpArg {
    block_index: u64,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize, Debug)]
enum NotifyError {
    Refunded { reason: String, block_index: Option<u64> },
    InvalidTransaction(String),
    Other { error_code: u64, error_message: String },
    Processing,
    TransactionTooOld(u64),
}

async fn notify_top_up(block_index: u64, canister_id: Principal) -> Result<u128, String> {
    let arg = NotifyTopUpArg { block_index, canister_id };

    let response: (Result<Nat, NotifyError>, ) = call(
        MAINNET_CYCLES_MINTING_CANISTER_ID,
        "notify_top_up",
        (arg, ),
    ).await.map_err(|(code, msg)| format!("Failed to call the CMC: {}: {}", code as u8, msg))?;

    match response.0 {
        Ok(cycles) => Ok(nat_to_u128(&cycles)),
        // The CMC sent the ICP back to the manager, so the caller has to be refunded.
        Err(NotifyError::Refunded { reason, .. }) =>
            Err(format!("The CMC refunded the top up: {}", reason)),
        // Retriable: the ICP is already on the CMC account, only the notification is pending.
        Err(NotifyError::Processing) =>
            Err(format!("The CMC is still processing the top up, retry notify_top_up for block {}", block_index)),
        Err(other) => Err(format!("The CMC rejected the top up: {:?}", other)),
    }
}

async fn transfer_from_manager(to: AccountIdentifier, amount: u64, memo: Memo) -> Result<u64, String> {
    let args = TransferArgs {
        memo,
        amount: Tokens::from_e8s(amount),
        fee: DEFAULT_FEE,
        from_subaccount: None,
        to,
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, args)
        .await
        .map_err(|(code, msg)| format!("Failed to call the ledger: {}: {}", code as u8, msg))?
        .map_err(|e| format!("Ledger transfer failed: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICP: u64 = 100_000_000;

    #[test]
    fn split_takes_ten_percent_and_covers_both_fees() {
        let split = split_payment(ICP).unwrap();
        assert_eq!(split.to_protocol, 10_000_000);
        assert_eq!(split.to_cycles, ICP - 10_000_000 - 20_000);
        assert_eq!(split.to_cycles + split.to_protocol + 20_000, ICP);
    }

    #[test]
    fn split_rejects_a_price_below_the_fees() {
        assert!(split_payment(DEFAULT_FEE.e8s()).is_err());
    }

    #[test]
    fn one_icp_at_four_xdr_mints_four_trillion_cycles() {
        // 40_000 permyriad per ICP is 4 XDR, and 1 XDR is 10^12 cycles.
        assert_eq!(e8s_to_cycles(ICP, 40_000), 4_000_000_000_000);
    }

    #[test]
    fn the_cycles_part_of_one_icp_covers_a_vault() {
        // A vault costs 500B cycles plus the 100B creation fee.
        let split = split_payment(ICP).unwrap();
        assert!(e8s_to_cycles(split.to_cycles, 40_000) >= 600_000_000_000);
    }

    #[test]
    fn destination_accepts_both_an_account_and_a_principal() {
        let account = "4918c656ea851d74504c84fe61581ef7cc00b282d44aa61b4c2c079ed189314e";
        assert_eq!(parse_destination(account).unwrap().to_string(), account);

        let principal = "sgk26-7yaaa-aaaan-qaovq-cai";
        let expected = AccountIdentifier::new(
            &Principal::from_text(principal).unwrap(),
            &DEFAULT_SUBACCOUNT,
        );
        assert_eq!(parse_destination(principal).unwrap(), expected);

        assert!(parse_destination("not an address").is_err());
    }

    #[test]
    fn nat_conversions_saturate_instead_of_wrapping() {
        assert_eq!(nat_to_u64(&Nat::from(42u64)), 42);
        assert_eq!(nat_to_u128(&Nat::from(42u64)), 42);
        assert_eq!(nat_to_u64(&(Nat::from(u64::MAX) + Nat::from(1u64))), u64::MAX);
    }
}
