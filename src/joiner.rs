use crate::authenticators::authenticator::ClientAuthenticator;
use crate::messages::abort::{Abort, MESSAGE_TYPE_ABORT};
use crate::messages::challenge::{Challenge, MESSAGE_TYPE_CHALLENGE};
use crate::messages::hello::Hello;
use crate::messages::message::Message;
use crate::messages::types::{Error, Value};
use crate::messages::welcome::{MESSAGE_TYPE_WELCOME, Welcome};
use crate::serializers::serializer::Serializer;
use crate::types::SessionDetails;
use std::collections::HashMap;

type JoinerState = u8;

const JOINER_STATE_NONE: JoinerState = 0;
const JOINER_STATE_HELLO_SENT: JoinerState = 1;
const JOINER_STATE_AUTHENTICATE_SENT: JoinerState = 2;
const JOINER_STATE_JOINED: JoinerState = 3;

pub fn get_client_roles() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "caller".to_string(),
        Value::Dict({
            let mut inner = HashMap::new();
            inner.insert("features".to_string(), Value::Dict(HashMap::new()));
            inner
        }),
    );

    map.insert(
        "callee".to_string(),
        Value::Dict({
            let mut inner = HashMap::new();
            inner.insert("features".to_string(), Value::Dict(HashMap::new()));
            inner
        }),
    );

    map.insert(
        "publisher".to_string(),
        Value::Dict({
            let mut inner = HashMap::new();
            inner.insert("features".to_string(), Value::Dict(HashMap::new()));
            inner
        }),
    );

    map.insert(
        "subscriber".to_string(),
        Value::Dict({
            let mut inner = HashMap::new();
            inner.insert("features".to_string(), Value::Dict(HashMap::new()));
            inner
        }),
    );

    map
}

pub struct Joiner {
    state: JoinerState,
    realm: String,
    session_details: Option<SessionDetails>,

    serializer: Box<dyn Serializer>,
    authenticator: Box<dyn ClientAuthenticator>,
}

impl Joiner {
    pub fn new(realm: &str, serializer: Box<dyn Serializer>, authenticator: Box<dyn ClientAuthenticator>) -> Self {
        Joiner {
            realm: realm.to_string(),
            state: JOINER_STATE_NONE,
            session_details: None,
            serializer,
            authenticator,
        }
    }

    pub fn send_hello(&mut self) -> Result<Vec<u8>, Error> {
        let method: Vec<String> = vec![self.authenticator.auth_method()];

        let hello = Hello::new(
            self.realm.as_str(),
            self.authenticator.authid().as_str(),
            self.authenticator.auth_extra(),
            get_client_roles(),
            method,
        );

        let raw_bytes = self.serializer.serialize(&hello);
        self.state = JOINER_STATE_HELLO_SENT;
        raw_bytes
    }

    pub fn receive(&mut self, data: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        match self.serializer.deserialize(data) {
            Ok(msg_in) => match self.receive_message(msg_in) {
                Ok(Some(msg_out)) => match self.serializer.serialize(msg_out.as_ref()) {
                    Ok(msg_out) => Ok(Some(msg_out)),
                    Err(e) => Err(e),
                },
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },

            Err(e) => Err(Error::new(format!("failed to deserialize message: {e:?}"))),
        }
    }

    pub fn receive_message(&mut self, msg: Box<dyn Message>) -> Result<Option<Box<dyn Message>>, Error> {
        match msg.message_type() {
            MESSAGE_TYPE_WELCOME => {
                if self.state != JOINER_STATE_HELLO_SENT && self.state != JOINER_STATE_AUTHENTICATE_SENT {
                    Err(Error::new("received WELCOME when it was not expected"))
                } else {
                    let welcome = msg.as_any().downcast_ref::<Welcome>().unwrap();

                    let realm = if welcome.realm.is_empty() {
                        self.realm.clone()
                    } else {
                        welcome.realm.clone()
                    };
                    let details = SessionDetails::new(
                        welcome.session_id,
                        realm,
                        welcome.authid.clone(),
                        welcome.auth_role.clone(),
                        false,
                    );
                    self.session_details = Some(details);

                    self.state = JOINER_STATE_JOINED;
                    Ok(None)
                }
            }

            MESSAGE_TYPE_CHALLENGE => {
                if self.state != JOINER_STATE_HELLO_SENT {
                    Err(Error::new("received CHALLENGE when it was not expected"))
                } else {
                    let challenge = msg.as_any().downcast_ref::<Challenge>().unwrap();
                    match self.authenticator.authenticate(challenge) {
                        Ok(authenticate) => {
                            self.state = JOINER_STATE_AUTHENTICATE_SENT;
                            Ok(Some(Box::new(authenticate)))
                        }

                        Err(e) => Err(Error::new(format!("failed to authenticate: {e:?}"))),
                    }
                }
            }

            MESSAGE_TYPE_ABORT => {
                let abort = msg.as_any().downcast_ref::<Abort>().unwrap();
                Err(Error::new(abort.reason.to_string()))
            }

            _ => Err(Error::new(format!(
                "received unknown message type {:?}",
                msg.message_type()
            ))),
        }
    }

    pub fn session_details(&self) -> Result<Option<&SessionDetails>, Error> {
        if self.session_details.is_none() {
            Err(Error::new("session is not setup yet"))
        } else {
            Ok(self.session_details.as_ref())
        }
    }
}
