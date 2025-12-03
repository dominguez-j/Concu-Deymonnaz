use crate::proxy::Proxy;
use actix::Message;
use lib::messages::types::UpdateType;
use tokio::net::TcpStream;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum AdminCommand {
    UpdateEnterpriseLimit {
        limit: u32,
        update_type: UpdateType,
    },
    UpdateCardLimit {
        card_id: u32,
        limit: u32,
        update_type: UpdateType,
    },
    CardView {
        card_id: u32,
    },
    EnterpriseView {},
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetProxyAddr(pub actix::Addr<Proxy>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewConnection(pub TcpStream);
