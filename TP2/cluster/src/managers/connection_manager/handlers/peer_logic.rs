use crate::ConnectionManager;
use crate::Role;
use crate::create_internode_server_peer_from_connection_send;
use crate::managers::election_manager::handlers::election::CallElection;
use crate::managers::election_manager::handlers::peers::{EMUntrack, RegisterPeerEM};
use crate::managers::heartbeat_manager::handlers::tracking::{HBTrack, HBUntrack};
use crate::managers::transaction_manager::RegisterPeerTM;
use crate::server_peer::*;
use actix::prelude::*;
use actix_async_handler::async_handler;
use lib::utils::next_round;
use lib::{debug, elog, log};

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPeer {
    pub(crate) id: u32,
    pub(crate) address: Addr<ServerPeer>,
    pub(crate) role: Role,
}

#[derive(Message, Clone, Copy)]
#[rtype(result = "()")]
pub struct PeerDown {
    pub(crate) id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewPeerDetectedFromHB {
    pub(crate) peer_id: u32,
}

impl Handler<RegisterPeer> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: RegisterPeer, _: &mut Context<Self>) {
        let current_id = self.id;

        match msg.role {
            Role::Internode => {
                let existed = self
                    .active_peers
                    .insert(msg.id, msg.address.clone())
                    .is_some();
                let keys = self.active_peers.keys().cloned().collect::<Vec<_>>();
                self.log_registration(existed, current_id, msg.id, msg.role.clone(), keys);

                if let Some(em) = &self.em {
                    em.do_send(RegisterPeerEM { id: msg.id });
                }
                if let Some(tm) = &self.tm {
                    tm.do_send(RegisterPeerTM {
                        id: msg.id,
                        addr: msg.address.clone(),
                    });
                }
            }
            Role::Station => {
                let existed = self.active_stations.insert(msg.id, msg.address).is_some();
                let keys = self.active_stations.keys().cloned().collect::<Vec<_>>();
                self.log_registration(existed, current_id, msg.id, msg.role.clone(), keys);
            }
            Role::Proxy => {
                let existed = self.active_proxies.insert(msg.id, msg.address).is_some();
                let keys = self.active_proxies.keys().cloned().collect::<Vec<_>>();
                self.log_registration(existed, current_id, msg.id, msg.role.clone(), keys);
            }
        }

        if let Some(hb) = &self.hb {
            hb.do_send(HBTrack {
                id: msg.id,
                role: msg.role,
            });
        };
    }
}
impl Handler<PeerDown> for ConnectionManager {
    type Result = ();
    fn handle(&mut self, msg: PeerDown, _: &mut Context<Self>) {
        let current_id = self.id;
        if let Some(tm) = &self.tm {
            tm.do_send(msg.clone());
        }
        if let Some(addr_to_stop) = self.active_peers.remove(&msg.id) {
            addr_to_stop.do_send(ShutdownServer);
            log!(
                "[CONN MANAGER {current_id}] Peer {} DOWN -> removed",
                msg.id
            );
        }

        if let Some(hb) = &self.hb {
            hb.do_send(HBUntrack { id: msg.id });
        }

        if let Some(em) = &self.em {
            em.do_send(EMUntrack { id: msg.id });
            if let Some(leader) = self.leader
                && leader == msg.id
            {
                self.leader = None;
                let round = next_round();
                em.do_send(CallElection { round });
                log!(
                    "[CONN MANAGER {}] Leader Down, sending CallElection, Round: {round}",
                    self.id
                );
            }
        }
    }
}
#[allow(clippy::unused_unit)]
#[async_handler]
impl Handler<NewPeerDetectedFromHB> for ConnectionManager {
    type Result = ();

    async fn handle(
        &mut self,
        msg: NewPeerDetectedFromHB,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let current_id = self.id;
        let peer_id = msg.peer_id;

        if current_id <= peer_id || self.active_peers.contains_key(&peer_id) {
            debug!("[CONN MANAGER {current_id}] ignore dial to {peer_id} (policy or exists)");
        } else {
            let addr_cm = _ctx.address();
            let peers_to_connect = Some(vec![peer_id]);
            let num_nodes = self.cfg.num_nodes;

            let res = create_internode_server_peer_from_connection_send(
                addr_cm,
                self.tm.as_mut().unwrap().clone(),
                num_nodes,
                current_id,
                peers_to_connect,
            )
            .await;

            log!(
                "[CONN MANAGER {current_id}] Sending reconnection to peer {}",
                peer_id
            );
            if let Err(e) = res {
                elog!("[CONN MANAGER {current_id}] dial error: {e}");
            }
        }
    }
}
