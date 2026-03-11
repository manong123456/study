use anyhow::Result;
use base64::Engine;
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::EncodePrivateKey;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub id: i64,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtIssuer {
    encoding_key: EncodingKey,
    jwks_json: String,
}

impl JwtIssuer {
    pub fn new() -> Result<Self> {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let point = verifying_key.to_encoded_point(false);

        let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let x = b64.encode(point.x().unwrap());
        let y = b64.encode(point.y().unwrap());

        let jwks_json = serde_json::to_string(&json!({
            "keys": [{
                "kty": "EC",
                "crv": "P-256",
                "x": x,
                "y": y,
                "use": "sig",
                "alg": "ES256"
            }]
        }))?;

        let pkcs8_der = signing_key.to_pkcs8_der()
            .map_err(|e| anyhow::anyhow!("pkcs8 error: {}", e))?;
        let encoding_key = EncodingKey::from_ec_der(pkcs8_der.as_bytes());

        Ok(Self { encoding_key, jwks_json })
    }

    pub fn issue_token(&self, username: &str, user_id: i64) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as usize;
        let claims = Claims {
            sub: username.to_string(),
            id: user_id,
            iat: now,
            exp: now + 3 * 3600,
        };
        let token = encode(&Header::new(Algorithm::ES256), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn jwks_json(&self) -> &str {
        &self.jwks_json
    }
}
