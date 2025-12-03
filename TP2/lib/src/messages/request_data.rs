use crate::ipc::representable::Representable;
use crate::prelude::InternodeSelect;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct RequestData {
    pub from: u32,
    pub original_id: String,
    pub select: InternodeSelect,
    pub initial_leader: Option<u32>,
}

impl Representable for RequestData {}
