use crate::locks::{AcquireLock, NextLock};
use crate::managers::connection_manager::handlers::{PeerDown, SetNewLeader};
use crate::managers::lock_manager::Pending;
use crate::managers::transaction_manager::{RegisterPeerTM, TransactionManager};
use actix::prelude::*;
use lib::prelude::*;

impl Handler<SetNewLeader> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: SetNewLeader, ctx: &mut Self::Context) -> Self::Result {
        self.leader = Some(msg.peer_id);
        log!("[TM {}] Leader changed to {}", self.id, msg.peer_id);
        self.lock_manager.clear();
        self.updates.iter_mut().for_each(|(id, update)| {
            update.initial_leader = self.leader;
            let TransactionType::Update(u) = &update.transaction else {
                panic!(""); // Cannot happen
            };
            if let Some(leader_peer) = self.active_peers.get(&msg.peer_id) {
                leader_peer.do_send(AcquireLock {
                    from: self.id,
                    enterprise_id: u.get_enterprise_id(),
                    transaction_id: id.clone(),
                })
            } else {
                ctx.address().do_send(Pending {
                    owner: None,
                    owner_id: self.id,
                    enterprise_id: u.get_enterprise_id(),
                    transaction_id: id.clone(),
                    got_lock: false,
                })
            }
        });
    }
}

impl Handler<RegisterPeerTM> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: RegisterPeerTM, _: &mut Self::Context) -> Self::Result {
        self.active_peers.entry(msg.id).or_insert(msg.addr);
        trace!("[TM {}] Peer: {} registered", self.id, msg.id);
        let keys = self.active_peers.keys().cloned().collect::<Vec<_>>();
        trace!("[TM {}] Active peers: {:?}", self.id, keys);
    }
}

impl Handler<PeerDown> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: PeerDown, ctx: &mut Self::Context) -> Self::Result {
        log!("[TM {}] Peer down with id {}", self.id, msg.id);
        self.active_peers.remove(&msg.id);
        let granteds = self.lock_manager.get_locks_granted_to(msg.id);
        for granted in granteds {
            ctx.address().do_send(NextLock {
                enterprise_id: granted,
            })
        }
    }
}
