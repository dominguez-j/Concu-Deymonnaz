use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ipc::representable::Representable;

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct TransactionResult {
    transaction_id: String,
    result: bool,
}

impl TransactionResult {
    pub fn new(transaction_id: String, result: bool) -> Self {
        Self {
            transaction_id,
            result,
        }
    }
    pub fn transaction_id(&self) -> String {
        self.transaction_id.clone()
    }
    pub fn result(&self) -> bool {
        self.result
    }
}

impl Representable for TransactionResult {}
