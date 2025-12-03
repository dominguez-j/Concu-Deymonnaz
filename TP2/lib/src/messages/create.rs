use crate::ipc::representable::Representable;
use actix::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct Create {
    pub enterprise_id: u32,
    pub enterprise_limit: u32,
    pub card_limits: HashMap<u32, u32>,
}

impl Representable for Create {}
