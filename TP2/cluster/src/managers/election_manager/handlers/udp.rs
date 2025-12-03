use crate::managers::connection_manager::handlers::setters::SetNewLeader;
use crate::managers::election_manager::handlers::election::CallElection;
use crate::managers::election_manager::handlers::timeouts::TimeoutForWaitingLeader;
use crate::{ElectionManager, ElectionStatus};
use actix::{AsyncContext, Context, Handler};
use lib::prelude::ProtocolMessage;
use lib::trace;
use lib::udp_io::{SendUdp, UdpInbound};

impl Handler<UdpInbound> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: UdpInbound, ctx: &mut Context<Self>) {
        let protocol_msg = msg.msg;
        let from_addr = msg.from;
        match protocol_msg {
            ProtocolMessage::Election { from, round } => {
                if let Some(leader) = self.leader
                    && leader == self.id
                {
                    let data = ProtocolMessage::ImLeader {
                        from: self.id,
                        round,
                    };
                    self.udp_io.do_send(SendUdp {
                        data,
                        target: from_addr,
                    });
                    trace!(
                        "[EM {}] Sending msg ImLeader directly to peer: {from}, Round: {round}",
                        self.id
                    );
                    return;
                }

                trace!(
                    "[EM {}] Sending msg ElectionOk to peer: {from}, Round: {round}",
                    self.id
                );
                let data = ProtocolMessage::ElectionOk {
                    from: self.id,
                    round,
                };
                self.udp_io.do_send(SendUdp {
                    data,
                    target: from_addr,
                });

                if self.round < round {
                    trace!("[EM {}] Calling Election, with new Round: {round}", self.id);
                    ctx.address().do_send(CallElection { round });
                }
            }
            ProtocolMessage::ImLeader { from, round } => {
                if round < self.round {
                    trace!(
                        "[EM {}] Ignoring stale ImLeader from {} round={} < current_round={}",
                        self.id, from, round, self.round
                    );
                    return;
                }
                self.leader = Some(from);
                self.round = round;
                self.election_status = ElectionStatus::Idle;
                self.cm.do_send(SetNewLeader { peer_id: from });
                trace!(
                    "[EM {}] Receive msg ImLeader from: {from}, Round: {round}",
                    self.id
                );
            }
            ProtocolMessage::ElectionOk { from, round } => {
                if round != self.round {
                    trace!(
                        "[EM {}] Ignoring ElectionOk from {} round={} != current_round={}",
                        self.id, from, round, self.round
                    );
                    return;
                }
                match self.election_status {
                    ElectionStatus::WaitingOk { round: r } if r == round => {
                        self.election_status = ElectionStatus::WaitingLeader { round };
                        ctx.address().do_send(TimeoutForWaitingLeader { round });
                        trace!(
                            "[EM {}] Receive ElectionOk from: {from}, Round: {round}",
                            self.id
                        );
                    }
                    _ => {
                        trace!(
                            "[EM {}] Ignoring ElectionOk from {} round={} (status={:?})",
                            self.id, from, round, self.election_status
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
