use crate::authenticators::authenticator::ClientAuthenticator;
use crate::messages::authenticate::Authenticate;
use crate::messages::challenge::Challenge;
use crate::messages::types::{Error, Value};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
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
        if let Some(Value::Str(test)) = challenge.extra.get("challenge") {
            match sign_cra_challenge(test, self.secret.as_bytes()) {
                Ok(signature) => Ok(Authenticate {
                    signature,
                    extra: self.extra.clone(),
                }),
                Err(err) => Err(Error::new(format!("{err}"))),
            }
        } else {
            Err(Error::new("challenge missing in auth extra or is none"))
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
