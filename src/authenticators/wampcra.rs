use crate::authenticators::authenticator::ClientAuthenticator;
use crate::messages::authenticate::Authenticate;
use crate::messages::challenge::Challenge;
use crate::messages::types::{Error, Value};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WAMPCRAAuthenticator {
    authid: String,
    secret: String,
    extra: HashMap<String, Value>,
}

impl ClientAuthenticator for WAMPCRAAuthenticator {
    fn auth_method(&self) -> String {
        "wampcra".to_string()
    }

    fn authid(&self) -> String {
        self.authid.clone()
    }

    fn auth_extra(&self) -> HashMap<String, Value> {
        self.extra.clone()
    }

    fn authenticate(&self, challenge: &Challenge) -> Result<Authenticate, Error> {
        let challenge_hex = match challenge.extra.get("challenge") {
            Some(Value::Str(s)) => s.to_string(),
            _ => return Err(Error::new("challenge must be a string")),
        };

        if challenge.extra.contains_key("salt")
            && challenge.extra.contains_key("iterations")
            && challenge.extra.contains_key("keylen")
        {
            let salt = match challenge.extra.get("salt") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err(Error::new("salt must be a string")),
            };

            let iterations = match challenge.extra.get("iterations") {
                Some(Value::Int(s)) => s,
                _ => return Err(Error::new("iterations must be an int")),
            };

            let keylen = match challenge.extra.get("keylen") {
                Some(Value::Int(s)) => s,
                _ => return Err(Error::new("keylen must be an int")),
            };

            let iterations: u32 = (*iterations)
                .try_into()
                .map_err(|_| Error::new("Invalid value for iterations: must be positive"))?;

            let keylen: usize = (*keylen)
                .try_into()
                .map_err(|_| Error::new("Invalid value for keylen: must be positive"))?;

            let key = derive_wamp_cra_key(self.secret.as_str(), salt.as_str(), iterations, keylen)
                .map_err(|e| Error::new(e.to_string()))?;

            let signature =
                sign_cra_challenge(challenge_hex.as_str(), key.as_bytes()).map_err(|e| Error::new(e.to_string()))?;

            Ok(Authenticate {
                signature,
                extra: self.extra.clone(),
            })
        } else {
            let signature = sign_cra_challenge(challenge_hex.as_str(), self.secret.as_bytes())
                .map_err(|e| Error::new(e.to_string()))?;

            Ok(Authenticate {
                signature,
                extra: self.extra.clone(),
            })
        }
    }
}

impl WAMPCRAAuthenticator {
    pub fn new(authid: &str, secret: &str, extra: HashMap<String, Value>) -> Self {
        WAMPCRAAuthenticator {
            authid: authid.to_string(),
            secret: secret.to_string(),
            extra,
        }
    }
}

// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

fn sign_cra_challenge(challenge: &str, key: &[u8]) -> Result<String, Error> {
    match HmacSha256::new_from_slice(key) {
        Ok(mut mac) => {
            mac.update(challenge.as_bytes());
            let signature = mac.finalize().into_bytes();
            Ok(general_purpose::STANDARD.encode(signature))
        }
        Err(err) => Err(Error::new(format!("Failed to generate mac {err}"))),
    }
}

pub fn derive_wamp_cra_key(secret: &str, salt: &str, iterations: u32, keylen: usize) -> Result<String, Error> {
    let mut derived_key = vec![0u8; keylen];

    pbkdf2_hmac::<Sha256>(secret.as_bytes(), salt.as_bytes(), iterations, &mut derived_key);
    Ok(general_purpose::STANDARD.encode(&derived_key))
}
