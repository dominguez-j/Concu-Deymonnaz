use crate::managers::heartbeat_manager::{HeartbeatManager, PeerState};
use actix::{Context, Handler, Message};
use lib::constants::general::{INTERNODE_BASE_PORT, PROXY_BASE_PORT, STATION_BASE_PORT};
use lib::prelude::*;
use lib::roles::Role;
use lib::utils::connection_socket_addr;
use std::time::Instant;

#[derive(Message)]
#[rtype(result = "()")]
pub struct HBTrack {
    pub(crate) id: u32,
    pub(crate) role: Role,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HBUntrack {
    pub(crate) id: u32,
}

/// Recibe los datos necesario (Id y Rol) para que el HeartBeat realize el respectivo seguimiento o
/// envío de mensajes de tipo HbMsg hacia o desde estas entidades (Internode, Station, Proxy) según corresponda
impl Handler<HBTrack> for HeartbeatManager {
    type Result = ();
    fn handle(&mut self, msg: HBTrack, _: &mut Context<Self>) {
        if msg.role == Role::Internode && msg.id == self.id {
            return;
        }
        match msg.role {
            Role::Internode => {
                let addr = connection_socket_addr(INTERNODE_BASE_PORT, msg.id);
                self.peers
                    .entry(msg.id)
                    .and_modify(|st| {
                        st.udp_addr = addr;
                        st.misses = 0;
                        st.last_seen = Instant::now();
                    })
                    .or_insert(PeerState {
                        udp_addr: addr,
                        misses: 0,
                        last_seen: Instant::now(),
                    });
                trace!("[HB {}] track internode {}", self.id, msg.id);
            }
            Role::Station => {
                let addr = connection_socket_addr(STATION_BASE_PORT, msg.id);
                self.stations.entry(msg.id).or_insert(addr);
                trace!("[HB {}] track station{}", self.id, msg.id);
            }
            Role::Proxy => {
                let addr = connection_socket_addr(PROXY_BASE_PORT, msg.id);
                self.proxies.entry(msg.id).or_insert(addr);
                trace!("[HB {}] track proxy{}", self.id, msg.id);
            }
        }
    }
}
impl Handler<HBUntrack> for HeartbeatManager {
    type Result = ();
    fn handle(&mut self, msg: HBUntrack, _: &mut Context<Self>) {
        if self.peers.remove(&msg.id).is_some() {
            trace!("[HB {}] untrack {}", self.id, msg.id);
        }
    }
}
