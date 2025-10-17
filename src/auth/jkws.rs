use anyhow;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, errors::ErrorKind, EncodingKey, Header};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Scope {
    Admin,
    User,
    Backoffice,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub token_type: TokenType,
    pub scope: Vec<Scope>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub token_type: TokenType,
    pub jti: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jwk {
    pub alg: String,
    pub e: String,
    pub kid: String,
    pub kty: String,
    pub n: String,
    #[serde(rename = "use")]
    pub r#use: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct AuthService {
    private_key: String,
    public_key: String, // Optional public key for RS256
}

impl AuthService {
    pub fn new(private_key: String, _jwt_expiration_hours: u64, public_key: String) -> Self {
        AuthService {
            private_key,
            public_key,
        }
    }

    /// Initialize AuthService with environment variables or generate new keys
    /// If JWT_PRIVATE_KEY is provided, it will be used. If JWT_PUBLIC_KEY is also provided,
    /// it will be used; otherwise, the public key will be extracted from the private key.
    /// If no keys are provided, new keys will be generated and saved to files.
    /// In all cases, public_keys.json will be created for compatibility.
    pub fn from_config(config: &crate::Args) -> anyhow::Result<Self> {
        match &config.private_key {
            Some(private_key) => {
                println!("🔑 Using JWT private key from environment variables");

                // Convert literal \n characters to actual newlines for PEM format
                let private_key = private_key.replace("\\n", "\n");

                // If public key is also provided, use it; otherwise extract from private key
                let public_key = match &config.public_key {
                    Some(public_key) => {
                        println!("🔑 Using JWT public key from environment variables");
                        public_key.replace("\\n", "\n")
                    }
                    None => {
                        println!("🔑 Extracting public key from private key");
                        Self::extract_public_key_from_private(&private_key)?
                    }
                };

                // Create public_keys.json for compatibility
                Self::create_public_keys_json(&public_key)?;

                Ok(AuthService::new(
                    private_key,
                    config.jwt_expiration_hours,
                    public_key,
                ))
            }
            _ => {
                println!(
                    "🔑 JWT keys not provided via environment variables, generating new keys..."
                );
                let keys = crate::misc::keypair::generate_key_pair()?;
                Ok(AuthService::new(
                    keys.private_key,
                    config.jwt_expiration_hours,
                    keys.public_key,
                ))
            }
        }
    }

    /// Extract public key from private key
    fn extract_public_key_from_private(private_key_pem: &str) -> anyhow::Result<String> {
        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let public_key_pem = public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        Ok(public_key_pem)
    }

    /// Create public_keys.json file for compatibility
    fn create_public_keys_json(public_key_pem: &str) -> anyhow::Result<()> {
        // Parse the public key from PEM format
        let public_key = rsa::RsaPublicKey::from_public_key_pem(public_key_pem)?;

        // Extract the modulus (n) and exponent (e) from the RSA public key
        let n = URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be());

        // Create the JWKS structure
        let jwks = serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "kid": "exchange_api_key_1",
                "use": "sig",
                "alg": "RS256",
                "n": n,
                "e": e
            }]
        });

        // Write to public_keys.json
        std::fs::write("public_keys.json", serde_json::to_string_pretty(&jwks)?)?;
        println!("📄 Public key saved to: public_keys.json");

        Ok(())
    }

    pub fn gen_access_token(
        &self,
        user_id: Uuid,
        email: String,
        scopes: Vec<Scope>,
    ) -> Result<String, ErrorKind> {
        let now = Utc::now();
        // Access tokens have short lifespan (15 minutes)
        let expiration = now + Duration::minutes(15);

        let claims = AccessTokenClaims {
            sub: user_id.to_string(),
            email,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "exchange_api".to_string(),
            token_type: TokenType::Access,
            scope: scopes,
        };

        // Use RS256 algorithm with private key for signing
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("exchange_api_key_1".to_string());

        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(self.private_key.as_bytes()).map_err(|e| e.into_kind())?,
        )
        .map_err(|e| e.into_kind())
    }

    pub fn gen_refresh_token(
        &self,
        user_id: Uuid,
        email: String,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        // Refresh tokens have long lifespan (30 days)
        let expiration = now + Duration::days(30);

        let claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            email,
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "exchange_api".to_string(),
            token_type: TokenType::Refresh,
            jti: Uuid::new_v4().to_string(), // Unique identifier for refresh token
        };

        // Use RS256 algorithm with private key for signing
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("exchange_api_key_1".to_string());

        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(self.private_key.as_bytes())?,
        )
    }

    pub fn gen_token_pair(
        &self,
        user_id: Uuid,
        email: String,
        scopes: Vec<Scope>,
    ) -> Result<(String, String), jsonwebtoken::errors::Error> {
        let access_token = self.gen_access_token(user_id, email.clone(), scopes)?;
        let refresh_token = self.gen_refresh_token(user_id, email)?;
        Ok((access_token, refresh_token))
    }

    pub fn verify_token(
        &self,
        token: &str,
    ) -> Result<AccessTokenClaims, jsonwebtoken::errors::Error> {
        // For backward compatibility, verify as access token
        let access_claims = self.verify_access_token(token)?;
        Ok(AccessTokenClaims {
            sub: access_claims.sub,
            email: access_claims.email,
            exp: access_claims.exp,
            iat: access_claims.iat,
            iss: access_claims.iss,
            token_type: access_claims.token_type,
            scope: access_claims.scope,
        })
    }

    /// Check if the token has admin scope
    pub fn has_admin_scope(&self, token: &str) -> Result<bool, jsonwebtoken::errors::Error> {
        let claims = self.verify_token(token)?;
        Ok(claims.scope.contains(&Scope::Admin))
    }

    /// Check if the token has a specific scope
    pub fn has_scope(
        &self,
        token: &str,
        required_scope: Scope,
    ) -> Result<bool, jsonwebtoken::errors::Error> {
        let claims = self.verify_token(token)?;
        Ok(claims.scope.contains(&required_scope))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, ErrorKind> {
        // Use RS256 algorithm with public key for verification
        let mut validation = jsonwebtoken::Validation::default();
        validation.algorithms = vec![jsonwebtoken::Algorithm::RS256];

        let decoded = jsonwebtoken::decode::<AccessTokenClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_rsa_pem(self.public_key.as_bytes())
                .map_err(|e| e.into_kind())?,
            &validation,
        )
        .map_err(|e| e.into_kind())?;

        // Verify it's an access token
        if decoded.claims.token_type != TokenType::Access {
            return Err(ErrorKind::InvalidToken);
        }

        Ok(decoded.claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, ErrorKind> {
        // Use RS256 algorithm with public key for verification
        let mut validation = jsonwebtoken::Validation::default();
        validation.algorithms = vec![jsonwebtoken::Algorithm::RS256];

        let decoded = jsonwebtoken::decode::<RefreshTokenClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_rsa_pem(self.public_key.as_bytes())
                .map_err(|e| e.into_kind())?,
            &validation,
        )
        .map_err(|e| e.into_kind())?;

        // Verify it's a refresh token
        if decoded.claims.token_type != TokenType::Refresh {
            return Err(ErrorKind::InvalidToken);
        }

        Ok(decoded.claims)
    }

    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        scopes: Vec<Scope>,
    ) -> Result<String, ErrorKind> {
        // Verify the refresh token
        let refresh_claims = self.verify_refresh_token(refresh_token)?;

        // Generate a new access token using the refresh token claims
        let user_id = Uuid::parse_str(&refresh_claims.sub).map_err(|_| ErrorKind::InvalidToken)?;

        self.gen_access_token(user_id, refresh_claims.email, scopes)
    }

    /// Generate JWKS (JSON Web Key Set) from the public key
    pub fn generate_jwks(&self) -> anyhow::Result<Jwks> {
        // Parse the public key from PEM format
        let public_key = rsa::RsaPublicKey::from_public_key_pem(&self.public_key)?;

        // Extract the modulus (n) and exponent (e) from the RSA public key
        let n = URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be());

        // Create the JWK
        let jwk = Jwk {
            alg: "RS256".to_string(),
            e,
            kid: "exchange_api_key_1".to_string(),
            kty: "RSA".to_string(),
            n,
            r#use: "sig".to_string(),
        };

        // Create the JWKS
        let jwks = Jwks { keys: vec![jwk] };

        Ok(jwks)
    }
}
