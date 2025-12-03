use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::managers::connection_manager::ConnectionManager;
use crate::managers::connection_manager::handlers::*;
use actix::{Actor, Addr, AsyncContext, Context};
use lib::config::Config;
use lib::constants::general::INTERNODE_BASE_PORT;
use lib::messages::protocol::ProtocolMessage;
use lib::trace;
use lib::udp_io::{SendUdp, UdpIO, UdpInbound, UdpSubscribe};
use lib::utils::{connection_socket_addr, now_ms};

const HB_INTERVAL_MS: u64 = 1000; // cada cuánto mandamos PING

pub struct PeerState {
    pub(crate) last_seen: Instant,
    pub(crate) misses: u32,
    pub(crate) udp_addr: SocketAddr,
}

pub struct HeartbeatManager {
    pub(crate) id: u32,
    cfg: Config,
    discovery_rounds_left: u32,
    seed_peers: Vec<SocketAddr>,
    udp_io: Addr<UdpIO>,
    pub(crate) peers: HashMap<u32, PeerState>,
    pub(crate) stations: HashMap<u32, SocketAddr>,
    pub(crate) proxies: HashMap<u32, SocketAddr>,
    pub(crate) cm: Addr<ConnectionManager>,
}

impl HeartbeatManager {
    pub(crate) fn new(
        id: u32,
        cm: Addr<ConnectionManager>,
        udp_io: Addr<UdpIO>,
        cfg: Config,
    ) -> Self {
        let init_num_nodes = cfg.num_nodes;
        let seed_peers = (1..=init_num_nodes)
            .filter(|&peers_id| peers_id > id)
            .map(|id| connection_socket_addr(INTERNODE_BASE_PORT, id))
            .collect::<Vec<_>>();

        let discovery_rounds_left = cfg.discovery_rounds;

        Self {
            id,
            cfg,
            discovery_rounds_left,
            seed_peers,
            peers: HashMap::new(),
            stations: HashMap::new(),
            proxies: HashMap::new(),
            cm,
            udp_io,
        }
    }
}

impl Actor for HeartbeatManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!(
            "[HB {}] started on UDP {}",
            self.id,
            INTERNODE_BASE_PORT + self.id
        );

        let recipient = ctx.address().recipient::<UdpInbound>();
        self.udp_io.do_send(UdpSubscribe { addr: recipient });

        ctx.run_later(Duration::from_millis(0), |act, _| {
            send_discovery_round(act);
        });

        ctx.run_interval(Duration::from_millis(HB_INTERVAL_MS), |actor, _| {
            send_discovery_round(actor);

            let hb_msg = ProtocolMessage::HbMsg {
                from: actor.id,
                ts_ms: now_ms(),
            };
            // /Pings proxies
            for &addr in actor.proxies.values() {
                actor.udp_io.do_send(SendUdp {
                    target: addr,
                    data: hb_msg.clone(),
                });
            }
            // /Pings stations
            for &addr in actor.stations.values() {
                actor.udp_io.do_send(SendUdp {
                    target: addr,
                    data: hb_msg.clone(),
                });
            }
            // /Pings peers
            for (_id, st) in actor.peers.iter_mut() {
                actor.udp_io.do_send(SendUdp {
                    target: st.udp_addr,
                    data: hb_msg.clone(),
                });
                st.misses = st.misses.saturating_add(1);
            }

            let hb_misses_allowed = actor.cfg.hb_misses_allowed;

            let downs: Vec<u32> = actor
                .peers
                .iter()
                .filter_map(|(id, st)| (st.misses >= hb_misses_allowed).then_some(*id))
                .collect();
            for id in downs {
                trace!(
                    "[HB {}] peer {} SUSPECTED DOWN (misses >= {})",
                    actor.id, id, hb_misses_allowed
                );
                actor.peers.remove(&id);
                actor.cm.do_send(PeerDown { id });
            }
        });
    }
}
/// Envía un Ping de descubrimiento a los internode peers y estaciones, una vez que se inicie el
/// programa, ya sea porque se ejecuta por primera vez el programa o porque el nodo cayó y volvió
/// a estar activo
/// La cantidad de veces que se envíe un ping de descubrimiento dependerá de la configuración inicial
fn send_discovery_round(act: &mut HeartbeatManager) {
    if act.discovery_rounds_left == 0 {
        return;
    }
    let hb_msg = ProtocolMessage::HbMsg {
        from: act.id,
        ts_ms: now_ms(),
    };

    for &addr in &act.seed_peers {
        act.udp_io.do_send(SendUdp {
            target: addr,
            data: hb_msg.clone(),
        });
        trace!("[HB {}] peers discovery -> {}", act.id, addr);
    }

    for &addr in act.stations.values() {
        act.udp_io.do_send(SendUdp {
            target: addr,
            data: hb_msg.clone(),
        });
        trace!("[HB {}] stations discovery -> {}", act.id, addr);
    }

    act.discovery_rounds_left = act.discovery_rounds_left.saturating_sub(1);
}
