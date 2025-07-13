use crate::authenticators::authenticator::ClientAuthenticator;
use crate::messages::authenticate::Authenticate;
use crate::messages::challenge::Challenge;
use crate::messages::types::{Error, Value};
use ed25519_dalek::{Signer, SigningKey};
use hex::{FromHex, ToHex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CryptoSignAuthenticator {
    authid: String,
    private_key: SigningKey,
    extra: HashMap<String, Value>,
}

impl ClientAuthenticator for CryptoSignAuthenticator {
    fn auth_method(&self) -> String {
        "cryptosign".to_string()
    }

    fn authid(&self) -> String {
        self.authid.clone()
    }

    fn auth_extra(&self) -> HashMap<String, Value> {
        self.extra.clone()
    }

    fn authenticate(&self, challenge: &Challenge) -> Result<Authenticate, Error> {
        if let Some(Value::Str(challenge_hex)) = challenge.extra.get("challenge") {
            match sign_crypto_sign_challenge(challenge_hex, &self.private_key) {
                Ok(signed) => Ok(Authenticate {
                    signature: signed,
                    extra: HashMap::new(),
                }),
                Err(err) => Err(Error::new(format!("failed to sign challenge: {err}"))),
            }
        } else {
            Err(Error::new("challenge missing in authextra or is none"))
        }
    }
}

impl CryptoSignAuthenticator {
    pub fn try_new(authid: &str, private_key: &str, extra: HashMap<String, Value>) -> Result<Self, Error> {
        match signing_key_from_hex(private_key) {
            Ok(key) => {
                let pub_key: String = key.verifying_key().encode_hex();
                let mut cloned = extra.clone();
                cloned.insert("pubkey".to_string(), Value::Str(pub_key));
                Ok(CryptoSignAuthenticator {
                    authid: authid.to_string(),
                    private_key: key,
                    extra: cloned,
                })
            }
            Err(err) => Err(Error::new(format!("{err}"))),
        }
    }
}

fn signing_key_from_hex(hex_str: &str) -> Result<SigningKey, Error> {
    match <[u8; 32]>::from_hex(hex_str) {
        Ok(bytes) => Ok(SigningKey::from_bytes(&bytes)),
        Err(e) => Err(Error::new(format!("invalid hex or wrong length: {e}"))),
    }
}

fn sign_crypto_sign_challenge(challenge: &str, private_key: &SigningKey) -> Result<String, Error> {
    match hex::decode(challenge) {
        Ok(challenge_raw) => {
            let signature = private_key.sign(&challenge_raw);
            let signed_hex = hex::encode(signature.to_vec());

            Ok(format!("{signed_hex}{challenge}"))
        }
        Err(e) => Err(Error::new(format!("failed to decode challenge hex: {e}"))),
    }
}
