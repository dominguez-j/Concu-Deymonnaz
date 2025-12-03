use actix::prelude::*;
use lib::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SelectWithId {
    pub(crate) transaction_id: String,
    pub(crate) select: Select,
}

impl SelectWithId {
    pub fn to_is(self) -> InternodeSelect {
        self.select.to_is(self.transaction_id)
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InternodeSelectWithId {
    pub(crate) transaction_id: String,
    pub(crate) select: InternodeSelect,
    pub(crate) initial_leader: Option<u32>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateWithId {
    pub(crate) initial_leader: Option<u32>,
    pub(crate) transaction_id: String,
    pub(crate) update: Update,
}

impl UpdateWithId {
    pub fn to_is(self) -> InternodeSelect {
        match self.update {
            Update::EnterpriseLimitUpdate { enterprise_id, .. } => {
                InternodeSelect::EnterpriseViewForUpdateFromInternode {
                    enterprise_id,
                    transaction_id: self.transaction_id.clone(),
                    initial_leader: self.initial_leader,
                }
            }
            Update::CardLimitUpdate {
                enterprise_id,
                card_id,
                ..
            } => InternodeSelect::CardViewForUpdateFromInternode {
                enterprise_id,
                card_id,
                transaction_id: self.transaction_id.clone(),
                initial_leader: self.initial_leader,
            },
            Update::Payment {
                enterprise_id,
                card_id,
                ..
            } => InternodeSelect::PaymentView {
                enterprise_id,
                card_id,
                transaction_id: self.transaction_id.clone(),
                initial_leader: self.initial_leader,
            },
        }
    }
}
