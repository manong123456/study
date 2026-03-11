//! JWT token issuance and JWKS distribution.
//!
//! Uses ES256 (ECDSA P-256) for signing. Generates a new key pair on startup.

use anyhow::Result;
use base64::Engine;
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::EncodePrivateKey;
use serde::{Serialize, Deserialize};
use serde_json::json;

/// JWT claims embedded in issued tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the username.
    pub sub: String,
    /// User ID from the database.
    pub id: i64,
    /// Expiration time (Unix timestamp).
    pub exp: usize,
    /// Issued-at time (Unix timestamp).
    pub iat: usize,
}

/// Issues and signs JWT tokens; provides JWKS for verification.
pub struct JwtIssuer {
    encoding_key: EncodingKey,
    jwks_json: String,
}

impl JwtIssuer {
    /// Creates a new JWT issuer with a freshly generated ECDSA P-256 key pair.
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

    /// Issues a signed JWT for the given user.
    ///
    /// # Arguments
    ///
    /// * `username` - The user's username (stored in `sub` claim).
    /// * `user_id` - The user's database ID (stored in `id` claim).
    ///
    /// # Returns
    ///
    /// A JWT string valid for 3 hours.
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

    /// Returns the JWKS (JSON Web Key Set) as a JSON string for public key distribution.
    pub fn jwks_json(&self) -> &str {
        &self.jwks_json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_issuer_creation() {
        let issuer = JwtIssuer::new();
        assert!(issuer.is_ok());
    }

    #[test]
    fn test_issue_token() {
        let issuer = JwtIssuer::new().unwrap();
        let token = issuer.issue_token("testuser", 1).unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_jwks_json_valid() {
        let issuer = JwtIssuer::new().unwrap();
        let jwks = issuer.jwks_json();
        let parsed: serde_json::Value = serde_json::from_str(jwks).unwrap();
        let keys = parsed["keys"].as_array().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0]["kty"], "EC");
    }

    #[test]
    fn test_different_tokens_for_different_users() {
        let issuer = JwtIssuer::new().unwrap();
        let token1 = issuer.issue_token("user1", 1).unwrap();
        let token2 = issuer.issue_token("user2", 2).unwrap();
        assert_ne!(token1, token2);
    }
}
