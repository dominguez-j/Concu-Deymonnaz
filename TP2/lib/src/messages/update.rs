use crate::ipc::representable::Representable;
use crate::messages::types::UpdateType;
use crate::prelude::{IdInterpreter, PaymentType};
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum Update {
    EnterpriseLimitUpdate {
        limit: u32,
        update_type: UpdateType,
        enterprise_id: u32,
    },
    CardLimitUpdate {
        card_id: u32,
        limit: u32,
        update_type: UpdateType,
        enterprise_id: u32,
    },
    Payment {
        payment_id: String,
        enterprise_id: u32,
        card_id: u32,
        transaction_type: PaymentType,
        cost: u32,
    },
}

impl Update {
    pub fn get_enterprise_id(&self) -> u32 {
        match self {
            Update::EnterpriseLimitUpdate { enterprise_id, .. } => *enterprise_id,
            Update::CardLimitUpdate { enterprise_id, .. } => *enterprise_id,
            Update::Payment { enterprise_id, .. } => *enterprise_id,
        }
    }

    pub fn get_update_id(&self) -> String {
        match self {
            Update::EnterpriseLimitUpdate { enterprise_id, .. } => {
                IdInterpreter::build_enterprise_limit_update_id(*enterprise_id)
            }
            Update::CardLimitUpdate {
                enterprise_id,
                card_id,
                ..
            } => IdInterpreter::build_spending_update_id(*enterprise_id, *card_id),
            Update::Payment { payment_id, .. } => payment_id.clone(),
        }
    }
}

impl Representable for Update {}
