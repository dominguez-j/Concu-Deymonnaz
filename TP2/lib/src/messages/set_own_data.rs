use crate::prelude::Representable;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UpdateRegister {
    EnterpriseLimit {
        limit: u32,
        usage: u32,
    },
    CardLimitOrUsage {
        card_id: u32,
        card_limit: u32,
        card_usage: u32,
        enterprise_limit: u32,
        enterprise_usage: u32,
    },
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct SetOwnData {
    pub enterprise_id: u32,
    pub data: UpdateRegister,
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct SendBroadcastOfSetOwnData {
    pub command: SetOwnData,
    pub initial_leader: Option<u32>,
    pub result: bool,
    pub do_broadcast: bool,
}

impl Representable for SetOwnData {}
