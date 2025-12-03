use crate::managers::election_manager::handlers::timeouts::TimeoutForWaitingOk;
use crate::{ElectionManager, ElectionStatus};
use actix::{AsyncContext, Context, Handler, Message};
use lib::prelude::ProtocolMessage;
use lib::trace;
use lib::udp_io::SendUdp;

#[derive(Message)]
#[rtype(result = "()")]
pub struct CallElection {
    pub(crate) round: u64,
}

impl Handler<CallElection> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: CallElection, ctx: &mut Context<Self>) {
        let round = msg.round;
        if round <= self.round {
            trace!(
                "[ELECTION {}] Ignoring CallElection for round {} <= current_round={}",
                self.id, round, self.round
            );
            return;
        }

        self.leader = None;
        self.round = round;
        self.election_status = ElectionStatus::WaitingOk { round };

        let peers = self
            .peers
            .iter()
            .filter(|(peer_id, _)| **peer_id > self.id)
            .map(|(peer_id, &addr)| (*peer_id, addr))
            .collect::<Vec<_>>();

        for (peer, addr) in peers.iter() {
            let data = ProtocolMessage::Election {
                from: self.id,
                round,
            };
            self.udp_io.do_send(SendUdp {
                data,
                target: *addr,
            });
            trace!(
                "[EM {}] Sending Election to peer {}. Round:{}",
                self.id, peer, round
            );
        }

        ctx.address().do_send(TimeoutForWaitingOk { round });
    }
}
