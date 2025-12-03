use crate::ipc::representable::Representable;
use crate::transaction::transaction_result::TransactionResult;
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardViewResponseData {
    pub card_usage: u32,
    pub enterprise_limit: Option<u32>,
    pub card_limit: Option<u32>,
    pub enterprise_usage: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnterpriseViewResponseData {
    pub usage: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentViewResponseInfo {
    pub enterprise_usage: u32,
    pub card_usage: u32,
    pub enterprise_limit: Option<u32>,
    pub card_limit: Option<u32>,
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum TransactionResponse {
    CardViewResponse {
        response: Option<CardViewResponseData>,
        enterprise_id: u32,
        card_id: u32,
        transaction_id: String,
    },
    EnterpriseViewResponse {
        response: Option<EnterpriseViewResponseData>,
        transaction_id: String,
    },
    TransactionResultResponse {
        result: TransactionResult,
        transaction_id: String,
    },
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum InternodeResponse {
    CardViewResponse {
        response: Option<CardViewResponseData>,
        enterprise_id: u32,
        card_id: u32,
        transaction_id: String,
    },
    EnterpriseViewResponse {
        response: Option<EnterpriseViewResponseData>,
        transaction_id: String,
    },
    CardViewForUpdateResponse {
        response: Option<CardViewResponseData>,
        transaction_id: String,
        initial_leader: Option<u32>,
    },
    EnterpriseViewForUpdateResponse {
        response: Option<EnterpriseViewResponseData>,
        transaction_id: String,
        initial_leader: Option<u32>,
    },
    PaymentViewResponse {
        response: Option<PaymentViewResponseInfo>,
        enterprise_id: u32,
        card_id: u32,
        transaction_id: String,
        initial_leader: Option<u32>,
    },
}

impl InternodeResponse {
    pub fn to_tr(self) -> Option<TransactionResponse> {
        match self {
            InternodeResponse::CardViewResponse {
                response,
                enterprise_id,
                card_id,
                transaction_id,
            } => Some(TransactionResponse::CardViewResponse {
                response,
                enterprise_id,
                card_id,
                transaction_id,
            }),
            InternodeResponse::EnterpriseViewResponse {
                response,
                transaction_id,
            } => Some(TransactionResponse::EnterpriseViewResponse {
                response,
                transaction_id,
            }),
            _ => None,
        }
    }

    pub fn get_original_transaction_id(&self) -> String {
        // "algo*internode*contador"
        // "algo", "interode*contador"
        self.get_transaction_id()
            .splitn(2, '*')
            .collect::<Vec<&str>>()[0]
            .parse()
            .unwrap()
    }

    pub fn get_transaction_id(&self) -> String {
        match self {
            InternodeResponse::CardViewResponse {
                transaction_id: id, ..
            } => id.clone(),
            InternodeResponse::EnterpriseViewResponse {
                transaction_id: id, ..
            } => id.clone(),
            InternodeResponse::EnterpriseViewForUpdateResponse {
                transaction_id: id, ..
            } => id.clone(),
            InternodeResponse::CardViewForUpdateResponse {
                transaction_id: id, ..
            } => id.clone(),
            InternodeResponse::PaymentViewResponse {
                transaction_id: id, ..
            } => id.clone(),
        }
    }

    pub fn get_initial_leader(&self) -> Option<u32> {
        match self {
            InternodeResponse::CardViewResponse { .. } => None,
            InternodeResponse::EnterpriseViewResponse { .. } => None,
            InternodeResponse::EnterpriseViewForUpdateResponse { initial_leader, .. } => {
                *initial_leader
            }
            InternodeResponse::CardViewForUpdateResponse { initial_leader, .. } => *initial_leader,
            InternodeResponse::PaymentViewResponse { initial_leader, .. } => *initial_leader,
        }
    }

    pub fn check_if_data_is_valid(&self) -> bool {
        match self {
            InternodeResponse::CardViewResponse { response, .. } => response.is_some(),
            InternodeResponse::EnterpriseViewResponse { response, .. } => response.is_some(),
            InternodeResponse::EnterpriseViewForUpdateResponse { response, .. } => {
                response.is_some()
            }
            InternodeResponse::CardViewForUpdateResponse { response, .. } => response.is_some(),
            InternodeResponse::PaymentViewResponse { response, .. } => response.is_some(),
        }
    }
}

impl Representable for TransactionResponse {}
