use crate::managers::election_manager::ElectionManager;
use crate::managers::heartbeat_manager::HeartbeatManager;
use crate::managers::transaction_manager::tm::TransactionManager;
use crate::server_peer::ServerPeer;
use actix::{Actor, Addr, Context};
use lib::config::Config;
use lib::roles::Role;
use lib::trace;
use std::collections::HashMap;

pub struct ConnectionManager {
    pub id: u32,
    pub leader: Option<u32>,
    pub cfg: Config,
    pub hb: Option<Addr<HeartbeatManager>>,
    pub em: Option<Addr<ElectionManager>>,
    pub tm: Option<Addr<TransactionManager>>,
    pub active_peers: HashMap<u32, Addr<ServerPeer>>,
    pub active_stations: HashMap<u32, Addr<ServerPeer>>,
    pub active_proxies: HashMap<u32, Addr<ServerPeer>>,
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;
}

impl ConnectionManager {
    pub(crate) async fn new(id: u32, cfg: Config) -> Self {
        Self {
            id,
            leader: None,
            cfg,
            hb: None,
            em: None,
            tm: None,
            active_peers: HashMap::new(),
            active_stations: HashMap::new(),
            active_proxies: HashMap::new(),
        }
    }

    pub fn log_registration(
        &mut self,
        existed: bool,
        current_id: u32,
        registered_id: u32,
        role: Role,
        keys: Vec<u32>,
    ) {
        if existed {
            trace!(
                "[CONN MANAGER {current_id}] Replaced {:?}Peer {}",
                role, registered_id
            );
        } else {
            trace!(
                "[CONN MANAGER {current_id}] Registered {:?}Peer {}",
                role, registered_id
            );
        }
        // /DEBUG
        trace!("[CONN MANAGER {current_id}] {:?}Peers = {:?}", role, keys);
    }
}
