use crate::ElectionManager;
use actix::{Context, Handler, Message};
use lib::constants::general::INTERNODE_BASE_PORT;
use lib::trace;
use lib::utils::connection_socket_addr;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPeerEM {
    pub id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EMUntrack {
    pub(crate) id: u32,
}

impl Handler<RegisterPeerEM> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: RegisterPeerEM, _: &mut Context<Self>) {
        let addr = connection_socket_addr(INTERNODE_BASE_PORT, msg.id);
        self.peers.entry(msg.id).or_insert(addr);
        trace!("[EM {}] Peer: {} registered", self.id, msg.id);
        let keys = self.peers.keys().cloned().collect::<Vec<_>>();
        trace!("[EM {}] Active peers: {:?}", self.id, keys);
    }
}

impl Handler<EMUntrack> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: EMUntrack, _: &mut Context<Self>) {
        if self.peers.remove(&msg.id).is_some() {
            trace!("[EM {}] untrack {}", self.id, msg.id);
        }
    }
}
