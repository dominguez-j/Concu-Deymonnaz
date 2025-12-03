use crate::ipc::representable::Representable;
use crate::messages::response::InternodeResponse;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum Select {
    CardView { enterprise_id: u32, card_id: u32 },
    EnterpriseView { enterprise_id: u32 },
}

impl Select {
    pub fn get_enterprise_id(&self) -> u32 {
        match self {
            Select::CardView { enterprise_id, .. } => *enterprise_id,
            Select::EnterpriseView { enterprise_id, .. } => *enterprise_id,
        }
    }

    pub fn to_is(self, transaction_id: String) -> InternodeSelect {
        match self {
            Select::CardView {
                enterprise_id,
                card_id,
            } => InternodeSelect::CardViewFromInternode {
                transaction_id,
                enterprise_id,
                card_id,
            },
            Select::EnterpriseView { enterprise_id } => {
                InternodeSelect::EnterpriseViewFromInternode {
                    transaction_id,
                    enterprise_id,
                }
            }
        }
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum InternodeSelect {
    CardViewFromInternode {
        transaction_id: String,
        enterprise_id: u32,
        card_id: u32,
    },
    EnterpriseViewFromInternode {
        transaction_id: String,
        enterprise_id: u32,
    },
    CardViewForUpdateFromInternode {
        transaction_id: String,
        enterprise_id: u32,
        card_id: u32,
        initial_leader: Option<u32>,
    },
    EnterpriseViewForUpdateFromInternode {
        transaction_id: String,
        enterprise_id: u32,
        initial_leader: Option<u32>,
    },
    PaymentView {
        transaction_id: String,
        enterprise_id: u32,
        card_id: u32,
        initial_leader: Option<u32>,
    },
}

impl InternodeSelect {
    pub fn get_enterprise_id(&self) -> u32 {
        match self {
            InternodeSelect::CardViewFromInternode { enterprise_id, .. } => *enterprise_id,
            InternodeSelect::EnterpriseViewFromInternode { enterprise_id, .. } => *enterprise_id,
            InternodeSelect::CardViewForUpdateFromInternode { enterprise_id, .. } => *enterprise_id,
            InternodeSelect::EnterpriseViewForUpdateFromInternode { enterprise_id, .. } => {
                *enterprise_id
            }
            InternodeSelect::PaymentView { enterprise_id, .. } => *enterprise_id,
        }
    }

    pub fn get_initial_leader(&self) -> Option<u32> {
        match self {
            InternodeSelect::CardViewFromInternode { .. } => None,
            InternodeSelect::EnterpriseViewFromInternode { .. } => None,
            InternodeSelect::CardViewForUpdateFromInternode { initial_leader, .. } => {
                *initial_leader
            }
            InternodeSelect::EnterpriseViewForUpdateFromInternode { initial_leader, .. } => {
                *initial_leader
            }
            InternodeSelect::PaymentView { initial_leader, .. } => *initial_leader,
        }
    }

    pub fn get_transaction_id(&self) -> String {
        match self {
            InternodeSelect::CardViewFromInternode { transaction_id, .. } => transaction_id.clone(),
            InternodeSelect::EnterpriseViewFromInternode { transaction_id, .. } => {
                transaction_id.clone()
            }
            InternodeSelect::CardViewForUpdateFromInternode { transaction_id, .. } => {
                transaction_id.clone()
            }
            InternodeSelect::EnterpriseViewForUpdateFromInternode { transaction_id, .. } => {
                transaction_id.clone()
            }
            InternodeSelect::PaymentView { transaction_id, .. } => transaction_id.clone(),
        }
    }

    pub fn update_transaction_id(&mut self, new_id: String) {
        let original = match self {
            InternodeSelect::CardViewFromInternode { transaction_id, .. } => transaction_id,
            InternodeSelect::EnterpriseViewFromInternode { transaction_id, .. } => transaction_id,
            InternodeSelect::CardViewForUpdateFromInternode { transaction_id, .. } => {
                transaction_id
            }
            InternodeSelect::EnterpriseViewForUpdateFromInternode { transaction_id, .. } => {
                transaction_id
            }
            InternodeSelect::PaymentView { transaction_id, .. } => transaction_id,
        };
        *original = new_id;
    }

    pub fn to_ir(self) -> Option<InternodeResponse> {
        match self {
            InternodeSelect::CardViewFromInternode {
                transaction_id,
                enterprise_id,
                card_id,
            } => Some(InternodeResponse::CardViewResponse {
                transaction_id,
                enterprise_id,
                card_id,
                response: None,
            }),
            InternodeSelect::CardViewForUpdateFromInternode {
                transaction_id,
                enterprise_id: _,
                card_id: _,
                initial_leader,
            } => Some(InternodeResponse::CardViewForUpdateResponse {
                transaction_id,
                response: None,
                initial_leader,
            }),
            InternodeSelect::EnterpriseViewForUpdateFromInternode {
                transaction_id,
                enterprise_id: _,
                initial_leader,
            } => Some(InternodeResponse::EnterpriseViewForUpdateResponse {
                transaction_id,
                response: None,
                initial_leader,
            }),
            InternodeSelect::EnterpriseViewFromInternode {
                transaction_id,
                enterprise_id: _,
            } => Some(InternodeResponse::EnterpriseViewResponse {
                transaction_id,
                response: None,
            }),
            InternodeSelect::PaymentView {
                transaction_id,
                enterprise_id,
                card_id,
                initial_leader,
            } => Some(InternodeResponse::PaymentViewResponse {
                transaction_id,
                enterprise_id,
                card_id,
                response: None,
                initial_leader,
            }),
        }
    }
}

impl Representable for Select {}
