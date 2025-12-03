use actix::prelude::*;
use lib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct CardView {
    pub(crate) transaction_id: String,
    pub(crate) card_id: u32,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct CardViewResponse {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
    pub(crate) card_id: u32,
    pub(crate) response: Option<CardViewResponseData>,
}

impl CardViewResponse {
    pub fn to_tr(self) -> TransactionResponse {
        TransactionResponse::CardViewResponse {
            card_id: self.card_id,
            enterprise_id: self.enterprise_id,
            transaction_id: self.transaction_id.clone(),
            response: self.response,
        }
    }

    pub fn to_is(self) -> InternodeSelect {
        InternodeSelect::CardViewFromInternode {
            enterprise_id: self.enterprise_id,
            card_id: self.card_id,
            transaction_id: self.transaction_id,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnterpriseView {
    pub(crate) transaction_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PaymentView {
    pub(crate) transaction_id: String,
    pub(crate) card_id: u32,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EnterpriseViewResponse {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
    pub(crate) data: Option<EnterpriseViewResponseData>,
}

impl EnterpriseViewResponse {
    pub fn to_tr(self) -> TransactionResponse {
        TransactionResponse::EnterpriseViewResponse {
            transaction_id: self.transaction_id.clone(),
            response: self.data,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnterpriseViewInternodeResponse {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
    pub(crate) data: Option<EnterpriseViewResponseData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentViewResponseData {
    pub enterprise_usage: u32,
    pub card_usage: u32,
    pub enterprise_limit: Option<u32>,
    pub card_limit: Option<u32>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PaymentViewResponse {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
    pub(crate) card_id: u32,
    pub(crate) data: Option<PaymentViewResponseData>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnterpriseLimitUpdate {
    pub(crate) initial_leader: Option<u32>,
    pub(crate) limit: u32,
    pub(crate) update_type: UpdateType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CardLimitUpdate {
    pub(crate) initial_leader: Option<u32>,
    pub(crate) limit: u32,
    pub(crate) card_id: u32,
    pub(crate) update_type: UpdateType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdatePayment {
    pub(crate) initial_leader: Option<u32>,
    pub(crate) id: String,
    pub(crate) card_id: u32,
    pub(crate) cost: u32,
}
