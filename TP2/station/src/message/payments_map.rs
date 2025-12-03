use actix::Message;
use lib::transaction::payment::Payment;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SavedPayments {
    payments: Vec<Payment>,
}

impl SavedPayments {
    pub fn new(payments: Vec<Payment>) -> Self {
        Self { payments }
    }
    pub fn payments(&self) -> &Vec<Payment> {
        &self.payments
    }
}
