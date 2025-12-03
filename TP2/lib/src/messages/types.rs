use crate::messages::select::InternodeSelect;
use crate::prelude::*;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum PaymentType {
    PaymentVerification,
    ForcePayment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UpdateType {
    Increment,
    Decrement,
    Set,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransactionType {
    Update(Update),
    Select(Select),
    InternodeSelect(InternodeSelect),
    Create(Create),
    RequestData(RequestData),
}
