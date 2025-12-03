use crate::ConnectionManager;
use crate::ElectionManager;
use crate::HeartbeatManager;
use crate::TransactionManager;
use actix::{Addr, Context, Handler, Message};
use lib::log;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetHeartbeatAddr {
    pub(crate) addr: Addr<HeartbeatManager>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetElectionManagerAddr {
    pub(crate) addr: Addr<ElectionManager>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetTransactionManagerAddr {
    pub(crate) addr: Addr<TransactionManager>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetNewLeader {
    pub(crate) peer_id: u32,
}

impl Handler<SetNewLeader> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: SetNewLeader, _ctx: &mut Self::Context) -> Self::Result {
        self.leader = Some(msg.peer_id);
        if let Some(tm) = &self.tm {
            tm.do_send(msg);
        }
        log!(
            "[CONN MANAGER {}] Set new leader, Peer: {}.",
            self.id,
            self.leader.unwrap()
        );
    }
}
impl Handler<SetHeartbeatAddr> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: SetHeartbeatAddr, _: &mut Context<Self>) {
        self.hb = Some(msg.addr);
    }
}

impl Handler<SetElectionManagerAddr> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: SetElectionManagerAddr, _: &mut Context<Self>) {
        self.em = Some(msg.addr);
    }
}

impl Handler<SetTransactionManagerAddr> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: SetTransactionManagerAddr, _: &mut Context<Self>) {
        self.tm = Some(msg.addr);
    }
}
