use crate::prelude::*;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub enum PumpReceiveMessageType {
    SetPumpId(SetPumpId),
    TransactionResult(TransactionResult),
}

impl Representable for PumpReceiveMessageType {}
