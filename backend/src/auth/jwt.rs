use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::auth::AuthUser;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub exp: i64,
}

#[derive(Serialize)]
struct JwtHeader<'a> {
    alg: &'a str,
    typ: &'a str,
}

fn b64_json<T: Serialize>(v: &T) -> Result<String, JwtError> {
    let json = serde_json::to_string(v).map_err(|_| JwtError::Encode)?;
    Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
}

fn sign(secret: &[u8], signing_input: &str) -> Result<Vec<u8>, JwtError> {
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| JwtError::Key)?;
    mac.update(signing_input.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn sign_token(secret: &str, user: &AuthUser, ttl_hours: i64) -> Result<String, JwtError> {
    let exp = (Utc::now() + Duration::hours(ttl_hours)).timestamp();
    let claims = Claims {
        sub: user.sub.to_string(),
        roles: user.roles.clone(),
        exp,
    };
    let header = JwtHeader {
        alg: "HS256",
        typ: "JWT",
    };
    let h = b64_json(&header)?;
    let p = b64_json(&claims)?;
    let signing_input = format!("{h}.{p}");
    let sig = sign(secret.as_bytes(), &signing_input)?;
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
    Ok(format!("{signing_input}.{sig_b64}"))
}

#[derive(Debug)]
pub enum JwtError {
    Format,
    Decode,
    Encode,
    Key,
    Signature,
    Expired,
    InvalidSub,
}

pub fn verify_token(secret: &str, token: &str) -> Result<AuthUser, JwtError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtError::Format);
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected = sign(secret.as_bytes(), &signing_input)?;
    let sig = URL_SAFE_NO_PAD
        .decode(parts[2].as_bytes())
        .map_err(|_| JwtError::Decode)?;
    if sig.len() != expected.len() || !bool::from(expected.as_slice().ct_eq(sig.as_slice())) {
        return Err(JwtError::Signature);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1].as_bytes())
        .map_err(|_| JwtError::Decode)?;
    let claims: Claims = serde_json::from_slice(&payload_bytes).map_err(|_| JwtError::Decode)?;
    if claims.exp < Utc::now().timestamp() {
        return Err(JwtError::Expired);
    }
    let sub = uuid::Uuid::parse_str(&claims.sub).map_err(|_| JwtError::InvalidSub)?;
    Ok(AuthUser {
        sub,
        roles: claims.roles,
    })
}
