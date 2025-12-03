use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Remove {
    transaction_id: String,
}

impl Remove {
    pub fn new(transaction_id: String) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
        }
    }
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
}
