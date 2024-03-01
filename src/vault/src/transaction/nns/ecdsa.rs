use candid::{CandidType, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext, TransformFunc,
};
use ic_cdk::api::time;
use ic_cdk::{id, print, trap};
use ic_transport_types::{to_request_id, EnvelopeContent};
use serde::Serialize;
use tracing::{error, info};
use sha2::{Digest, Sha256};
use k256::PublicKey;
use k256::pkcs8::EncodePublicKey;

pub fn get_key_id(is_local_dev_mode: bool) -> EcdsaKeyId {
    let key_name = if is_local_dev_mode { "dfx_test_key" } else { "key_1" };

    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(),
    }
}

pub struct CanisterEcdsaRequest {
    pub envelope_content: EnvelopeContent,
    pub request_url: String,
    pub public_key: Vec<u8>,
    pub key_id: EcdsaKeyId,
    pub this_canister_id: Principal,
}

pub async fn make_canister_call_via_ecdsa(request: CanisterEcdsaRequest) -> Result<String, String> {
    let body = match sign_envelope(request.envelope_content, request.public_key, request.key_id).await {
        Ok(bytes) => bytes,
        Err(error) => return Err(format!("Failed to sign envelope: {error:?}")),
    };

    let (response,) = ic_cdk::api::management_canister::http_request::http_request(
        CanisterHttpRequestArgument {
            url: request.request_url,
            max_response_bytes: Some(1024 * 1024), // 1 MB
            method: HttpMethod::POST,
            headers: vec![HttpHeader {
                name: "content-type".to_string(),
                value: "application/cbor".to_string(),
            }],
            body: Some(body),
            transform: Some(TransformContext {
                function: TransformFunc::new(request.this_canister_id, "transform_http_response".to_string()),
                context: Vec::new(),
            }),
        },
        100_000_000_000,
    )
        .await
        .map_err(|error| format!("Failed to make http request: {error:?}"))?;

    Ok(String::from_utf8(response.body).unwrap())
}

async fn sign_envelope(content: EnvelopeContent, public_key: Vec<u8>, key_id: EcdsaKeyId) -> CallResult<Vec<u8>> {
    let request_id = to_request_id(&content).unwrap();

    let signature = sign(key_id, &request_id.signable()).await?;

    let envelope = Envelope {
        content: content.clone(),
        sender_pubkey: Some(public_key),
        sender_sig: Some(signature.clone()),
    };

    let mut serialized_bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut serialized_bytes);
    serializer.self_describe().unwrap();
    envelope.serialize(&mut serializer).unwrap();

    info!(
        request_id = String::from(request_id),
        signature = hex::encode(signature),
        "Signed envelope"
    );

    Ok(serialized_bytes)
}
async fn sign(key_id: EcdsaKeyId, message: &[u8]) -> CallResult<Vec<u8>> {
    let message_hash = sha256(message);
    print(format!("message_hash: {:?}", key_id.name));
    match ic_cdk::api::management_canister::ecdsa::sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash: message_hash.to_vec(),
        derivation_path: Vec::new(),
        key_id,
    })
    .await
    {
        Ok(res) => Ok(res.0.signature),
        Err(error) => {
            error!(?error, "Error calling 'sign_with_ecdsa'");
            Err(error)
        }
    }
}

#[derive(Serialize)]
struct Envelope {
    content: EnvelopeContent,
    #[serde(with = "serde_bytes")]
    sender_pubkey: Option<Vec<u8>>,
    #[serde(with = "serde_bytes")]
    sender_sig: Option<Vec<u8>>,
}


pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

pub fn get_public_key_der(bytes: Vec<u8>) -> Vec<u8> {
    PublicKey::from_sec1_bytes(&bytes)
        .unwrap()
        .to_public_key_der()
        .unwrap()
        .to_vec()
}

pub fn get_principal(bytes: Vec<u8>) -> Principal {
    Principal::self_authenticating(get_public_key_der(bytes))
}

pub async fn prepare_canister_call_via_ecdsa<A: CandidType>(
    canister_id: Principal, method_name: String, args: A,) -> CanisterEcdsaRequest {
    let a =  EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(),
    };
    // let pk = get_public_key(a).await;
    //
    // let pkk = match pk {
    //     Ok(pk) => pk,
    //     Err(e) => {
    //         trap(&format!("Failed to get public key: {e:?}"));
    //     }
    // };

    if let Ok(public_key) = get_public_key(a).await {
        let envelope_content = EnvelopeContent::Call {
            nonce: None,
            ingress_expiry: time() + 5 * MINUTE_IN_MS * NANOS_PER_MILLISECOND,
            sender: get_principal(public_key.clone()),
            canister_id,
            method_name,
            arg: candid::encode_one(&args).unwrap(),
        };

        CanisterEcdsaRequest {
            envelope_content,
            request_url: format!("https://6ceb-79-153-174-101.ngrok-free.app/api/v2/canister/{canister_id}/call"),
            public_key: get_public_key_der(public_key.clone()),
            key_id: get_key_id(false),
            this_canister_id: id(),
        }
    } else {
        trap("Failed to get public key");
    }

}
pub type Milliseconds = u64;

pub const SECOND_IN_MS: Milliseconds = 1000;
pub const MINUTE_IN_MS: Milliseconds = SECOND_IN_MS * 60;
pub const HOUR_IN_MS: Milliseconds = MINUTE_IN_MS * 60;
pub const DAY_IN_MS: Milliseconds = HOUR_IN_MS * 24;
pub const WEEK_IN_MS: Milliseconds = DAY_IN_MS * 7;

pub const NANOS_PER_MILLISECOND: u64 = 1_000_000;




pub async fn get_public_key(key_id: EcdsaKeyId) -> CallResult<Vec<u8>> {
    match ic_cdk::api::management_canister::ecdsa::ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: Vec::new(),
        key_id,
    })
        .await
    {
        Ok(res) => Ok(res.0.public_key),
        Err(error) => {
            error!(?error, "Error calling 'ecdsa_public_key'");
            Err(error)
        }
    }
}
