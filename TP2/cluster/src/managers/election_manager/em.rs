use crate::ConnectionManager;
use crate::managers::election_manager::handlers::election::CallElection;
use actix::{Actor, Addr, AsyncContext, Context};
use lib::constants::general::{INTERNODE_BASE_PORT, WAITING_FIRST_ELECTION_TIMEOUT_S};
use lib::trace;
use lib::udp_io::{UdpIO, UdpInbound, UdpSubscribe};
use lib::utils::next_round;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub enum ElectionStatus {
    Idle, // /Es un nodo no líder
    WaitingOk { round: u64 },
    WaitingLeader { round: u64 },
    Leader { round: u64 },
}

pub struct ElectionManager {
    pub(crate) id: u32,
    pub(crate) leader: Option<u32>,
    pub(crate) election_status: ElectionStatus,
    /// Round más alto que se conoce (propio o de otro nodo)
    pub(crate) round: u64,
    pub(crate) udp_io: Addr<UdpIO>,
    pub(crate) cm: Addr<ConnectionManager>,
    pub(crate) peers: HashMap<u32, SocketAddr>,
}

impl ElectionManager {
    pub(crate) fn new(id: u32, udp_io: Addr<UdpIO>, cm: Addr<ConnectionManager>) -> Self {
        Self {
            id,
            leader: None,
            election_status: ElectionStatus::Idle,
            round: 0,
            udp_io,
            cm,
            peers: HashMap::new(),
        }
    }
}

impl Actor for ElectionManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!(
            "[EM {}] started on UDP {}",
            self.id,
            INTERNODE_BASE_PORT + self.id
        );
        let recipient = ctx.address().recipient::<UdpInbound>();
        self.udp_io.do_send(UdpSubscribe { addr: recipient });

        let round = next_round();
        ctx.run_later(
            Duration::from_secs(WAITING_FIRST_ELECTION_TIMEOUT_S),
            move |act, ctx| {
                ctx.address().do_send(CallElection { round });
                trace!(
                    "[EM {}] CallElection by first time on started. With Round: {round}",
                    act.id
                );
            },
        );
    }
}
