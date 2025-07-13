use crate::serializers::serializer::Serializer;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::messages::call::{Call, MESSAGE_TYPE_CALL};
use crate::messages::error::{Error, MESSAGE_TYPE_ERROR};
use crate::messages::event::{Event, MESSAGE_TYPE_EVENT};
use crate::messages::goodbye::MESSAGE_TYPE_GOODBYE;
use crate::messages::invocation::{Invocation, MESSAGE_TYPE_INVOCATION};
use crate::messages::message::Message;
use crate::messages::publish::{MESSAGE_TYPE_PUBLISH, Publish};
use crate::messages::published::{MESSAGE_TYPE_PUBLISHED, Published};
use crate::messages::register::{MESSAGE_TYPE_REGISTER, Register};
use crate::messages::registered::{MESSAGE_TYPE_REGISTERED, Registered};
use crate::messages::result::{MESSAGE_TYPE_RESULT, Result_};
use crate::messages::subscribe::{MESSAGE_TYPE_SUBSCRIBE, Subscribe};
use crate::messages::subscribed::{MESSAGE_TYPE_SUBSCRIBED, Subscribed};
use crate::messages::types::{Error as XError, Value};
use crate::messages::unregister::{MESSAGE_TYPE_UNREGISTER, Unregister};
use crate::messages::unregistered::{MESSAGE_TYPE_UNREGISTERED, Unregistered};
use crate::messages::unsubscribe::{MESSAGE_TYPE_UNSUBSCRIBE, Unsubscribe};
use crate::messages::unsubscribed::{MESSAGE_TYPE_UNSUBSCRIBED, Unsubscribed};
use crate::messages::validator::get_type;
use crate::messages::yield_::{MESSAGE_TYPE_YIELD, Yield};

#[derive(Debug)]
pub struct Session {
    serializer: Box<dyn Serializer>,

    call_requests: Mutex<HashSet<i64>>,
    register_requests: Mutex<HashSet<i64>>,
    registrations: Mutex<HashSet<i64>>,
    invocation_requests: Mutex<HashSet<i64>>,
    unregister_requests: Mutex<HashMap<i64, i64>>,

    publish_requests: Mutex<HashSet<i64>>,
    subscribe_requests: Mutex<HashSet<i64>>,
    subscriptions: Mutex<HashSet<i64>>,
    unsubscribe_requests: Mutex<HashSet<i64>>,
}

impl Session {
    pub fn new(serializer: Box<dyn Serializer>) -> Session {
        Session {
            serializer,

            call_requests: Default::default(),
            register_requests: Default::default(),
            registrations: Default::default(),
            invocation_requests: Default::default(),
            unregister_requests: Default::default(),

            publish_requests: Default::default(),
            subscribe_requests: Default::default(),
            subscriptions: Default::default(),
            unsubscribe_requests: Default::default(),
        }
    }

    pub fn send_message(&self, msg: Box<dyn Message>) -> Result<Vec<u8>, XError> {
        match self.serializer.serialize(msg.as_ref()) {
            Ok(data) => match msg.message_type() {
                MESSAGE_TYPE_CALL => {
                    let call = msg.as_any().downcast_ref::<Call>().unwrap();
                    let mut map = self.call_requests.lock().unwrap();
                    map.insert(call.request_id);
                    Ok(data)
                }

                MESSAGE_TYPE_YIELD => {
                    let yield_ = msg.as_any().downcast_ref::<Yield>().unwrap();
                    let mut map = self.invocation_requests.lock().unwrap();
                    map.remove(&yield_.request_id);
                    Ok(data)
                }

                MESSAGE_TYPE_REGISTER => {
                    let register = msg.as_any().downcast_ref::<Register>().unwrap();
                    let mut map = self.register_requests.lock().unwrap();
                    map.insert(register.request_id);
                    Ok(data)
                }

                MESSAGE_TYPE_UNREGISTER => {
                    let unregister = msg.as_any().downcast_ref::<Unregister>().unwrap();
                    let mut map = self.unregister_requests.lock().unwrap();
                    map.insert(unregister.request_id, unregister.registration_id);
                    Ok(data)
                }

                MESSAGE_TYPE_PUBLISH => {
                    let publish = msg.as_any().downcast_ref::<Publish>().unwrap();
                    if let Some(Value::Bool(acknowledge)) = publish.options.get("acknowledge") {
                        if *acknowledge {
                            let mut map = self.publish_requests.lock().unwrap();
                            map.insert(publish.request_id);
                        }
                    }

                    Ok(data)
                }

                MESSAGE_TYPE_SUBSCRIBE => {
                    let subscribe = msg.as_any().downcast_ref::<Subscribe>().unwrap();
                    let mut map = self.subscribe_requests.lock().unwrap();
                    map.insert(subscribe.request_id);
                    Ok(data)
                }

                MESSAGE_TYPE_UNSUBSCRIBE => {
                    let unsubscribe = msg.as_any().downcast_ref::<Unsubscribe>().unwrap();
                    let map = self.subscriptions.lock().unwrap();
                    if map.contains(&unsubscribe.subscription_id) {
                        let mut unsubmap = self.unsubscribe_requests.lock().unwrap();
                        unsubmap.insert(unsubscribe.request_id);
                        Ok(data)
                    } else {
                        Err(XError::new(format!(
                            "unsubscribe request for non existent subscription {}",
                            unsubscribe.subscription_id
                        )))
                    }
                }

                MESSAGE_TYPE_ERROR => {
                    let error = msg.as_any().downcast_ref::<Error>().unwrap();
                    if error.message_type != MESSAGE_TYPE_INVOCATION {
                        Err(XError::new(
                            "error message can only be sent for message_type=INVOCATION",
                        ))
                    } else {
                        let mut map = self.invocation_requests.lock().unwrap();
                        map.remove(&error.request_id);
                        Ok(data)
                    }
                }

                MESSAGE_TYPE_GOODBYE => Ok(data),

                _ => Err(XError::new(format!(
                    "send not supported for message of type {}",
                    get_type(&msg)
                ))),
            },

            Err(e) => Err(XError::new(format!("failed to serialize {e}"))),
        }
    }

    pub fn receive(&self, data: Vec<u8>) -> Result<Box<dyn Message>, XError> {
        match self.serializer.deserialize(data) {
            Ok(msg) => match self.receive_message(msg.as_ref()) {
                Ok(()) => Ok(msg),
                Err(err) => Err(XError::new(format!("failed to deserialize {err}"))),
            },
            Err(err) => Err(XError::new(format!("failed to deserialize {err}"))),
        }
    }

    pub fn receive_message(&self, msg: &dyn Message) -> Result<(), XError> {
        match msg.message_type() {
            MESSAGE_TYPE_RESULT => {
                let result = msg.as_any().downcast_ref::<Result_>().unwrap();
                let mut map = self.call_requests.lock().unwrap();
                if map.contains(&result.request_id) {
                    map.remove(&result.request_id);
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received RESULT for invalid request_id {}",
                        result.request_id
                    )))
                }
            }

            MESSAGE_TYPE_REGISTERED => {
                let registered = msg.as_any().downcast_ref::<Registered>().unwrap();
                let mut map = self.register_requests.lock().unwrap();
                if map.remove(&registered.request_id) {
                    let mut map = self.registrations.lock().unwrap();
                    map.insert(registered.registration_id);
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received REGISTERED for invalid request_id {}",
                        registered.request_id
                    )))
                }
            }

            MESSAGE_TYPE_UNREGISTERED => {
                let unregistered = msg.as_any().downcast_ref::<Unregistered>().unwrap();
                let mut map = self.unregister_requests.lock().unwrap();
                if let Some(reg_id) = map.remove(&unregistered.request_id) {
                    let mut regmap = self.registrations.lock().unwrap();
                    if regmap.take(&reg_id).is_some() {
                        Ok(())
                    } else {
                        Err(XError::new(format!(
                            "received UNREGISTERED for invalid registration_id {reg_id}"
                        )))
                    }
                } else {
                    Err(XError::new(format!(
                        "received UNREGISTERED for invalid request_id {}",
                        unregistered.request_id
                    )))
                }
            }

            MESSAGE_TYPE_INVOCATION => {
                let invocation = msg.as_any().downcast_ref::<Invocation>().unwrap();
                let map = self.registrations.lock().unwrap();
                if map.contains(&invocation.registration_id) {
                    let mut invmap = self.invocation_requests.lock().unwrap();
                    invmap.insert(invocation.request_id);
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received INVOCATION for invalid request_id {}",
                        invocation.request_id
                    )))
                }
            }

            MESSAGE_TYPE_PUBLISHED => {
                let published = msg.as_any().downcast_ref::<Published>().unwrap();
                let mut map = self.publish_requests.lock().unwrap();
                if map.contains(&published.request_id) {
                    map.remove(&published.request_id);
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received PUBLISHED for invalid request_id {}",
                        published.request_id
                    )))
                }
            }

            MESSAGE_TYPE_SUBSCRIBED => {
                let subscribed = msg.as_any().downcast_ref::<Subscribed>().unwrap();
                let map = self.subscribe_requests.lock().unwrap();
                if map.contains(&subscribed.request_id) {
                    let mut submap = self.subscriptions.lock().unwrap();
                    submap.insert(subscribed.subscription_id);
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received SUBSCRIBED for invalid request_id {}",
                        subscribed.request_id
                    )))
                }
            }

            MESSAGE_TYPE_UNSUBSCRIBED => {
                let unsubscribed = msg.as_any().downcast_ref::<Unsubscribed>().unwrap();
                let map = self.unsubscribe_requests.lock().unwrap();
                if let Some(id) = map.get(&unsubscribed.request_id) {
                    let mut submap = self.subscriptions.lock().unwrap();
                    if submap.take(id).is_some() {
                        Ok(())
                    } else {
                        Err(XError::new(format!(
                            "received UNSUBSCRIBED for invalid subscription_id {id}"
                        )))
                    }
                } else {
                    Err(XError::new(format!(
                        "received UNSUBSCRIBED for invalid request_id {}",
                        unsubscribed.request_id
                    )))
                }
            }

            MESSAGE_TYPE_EVENT => {
                let event = msg.as_any().downcast_ref::<Event>().unwrap();
                let map = self.subscriptions.lock().unwrap();
                if map.contains(&event.subscription_id) {
                    Ok(())
                } else {
                    Err(XError::new(format!(
                        "received EVENT for invalid subscription_id {}",
                        event.subscription_id
                    )))
                }
            }

            MESSAGE_TYPE_ERROR => {
                let error = msg.as_any().downcast_ref::<Error>().unwrap();
                match error.message_type {
                    MESSAGE_TYPE_CALL => {
                        let mut map = self.call_requests.lock().unwrap();
                        if map.remove(&error.request_id) {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid call request"))
                        }
                    }

                    MESSAGE_TYPE_REGISTER => {
                        let mut map = self.register_requests.lock().unwrap();
                        if map.remove(&error.request_id) {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid register request"))
                        }
                    }

                    MESSAGE_TYPE_UNREGISTER => {
                        let mut map = self.unregister_requests.lock().unwrap();
                        if map.remove(&error.request_id).is_some() {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid unregister request"))
                        }
                    }

                    MESSAGE_TYPE_SUBSCRIBE => {
                        let mut map = self.subscribe_requests.lock().unwrap();
                        if map.remove(&error.request_id) {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid subscribe request"))
                        }
                    }

                    MESSAGE_TYPE_UNSUBSCRIBE => {
                        let mut map = self.unsubscribe_requests.lock().unwrap();
                        if map.remove(&error.request_id) {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid unsubscribe request"))
                        }
                    }

                    MESSAGE_TYPE_PUBLISH => {
                        let mut map = self.publish_requests.lock().unwrap();
                        if map.remove(&error.request_id) {
                            Ok(())
                        } else {
                            Err(XError::new("received ERROR for invalid publish request"))
                        }
                    }

                    _ => Err(XError::new(format!(
                        "unknown error message type {:?}",
                        msg.message_type()
                    ))),
                }
            }

            MESSAGE_TYPE_GOODBYE => Ok(()),

            _ => Err(XError::new(format!(
                "received unexpected message type {:?}",
                msg.message_type()
            ))),
        }
    }
}
