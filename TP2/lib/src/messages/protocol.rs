use crate::ipc::representable::Representable;
use crate::messages::request_data::RequestData;
use crate::messages::response::{InternodeResponse, TransactionResponse};
use crate::messages::select::Select;
use crate::messages::set_own_data::SetOwnData;
use crate::messages::update::Update;
use crate::prelude::Create;
use crate::roles::Role;
use actix::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub enum ProtocolMessage {
    StartUp {
        from: u32,
        role: Role,
    },
    HbMsg {
        from: u32,
        ts_ms: u64,
    },
    FirstMsg {
        from: u32,
        data: String,
    },
    Election {
        from: u32,
        round: u64,
    },
    ElectionOk {
        from: u32,
        round: u64,
    },
    ImLeader {
        from: u32,
        round: u64,
    },
    EnterpriseCreate {
        enterprise_id: u32,
        enterprise_balance: u32,
        card_limits: HashMap<u32, u32>,
    },
    AcquireLock {
        from: u32,
        transaction_id: String,
        enterprise_id: u32,
    },
    ReleaseLock {
        transaction_id: String,
        enterprise_id: u32,
    },
    LockGranted {
        transaction_id: String,
        enterprise_id: u32,
    },

    Update(Update),
    Select(Select),
    TransactionResponse(TransactionResponse),
    InternodeResponse(InternodeResponse),
    RequestData(RequestData),
    InternodeCreate(Create),
    InternodeSetOwnData(SetOwnData),
}

impl Representable for ProtocolMessage {}
