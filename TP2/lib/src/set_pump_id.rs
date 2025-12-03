use crate::prelude::Representable;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SetPumpId {
    id: u32,
}

impl SetPumpId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Representable for SetPumpId {}
