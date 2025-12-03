use crate::managers::connection_manager::handlers::setters::SetNewLeader;
use crate::managers::election_manager::handlers::election::CallElection;
use crate::{ElectionManager, ElectionStatus};
use actix::{AsyncContext, Context, Handler, Message};
use lib::constants::general::{TIMEOUT_FOR_LEADER_MS, TIMEOUT_FOR_OK_MS};
use lib::prelude::ProtocolMessage;
use lib::trace;
use lib::udp_io::SendUdp;
use lib::utils::next_round;
use std::time::Duration;

#[derive(Message)]
#[rtype(result = "()")]
pub struct TimeoutForWaitingOk {
    pub(crate) round: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TimeoutForWaitingLeader {
    pub(crate) round: u64,
}

impl Handler<TimeoutForWaitingOk> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: TimeoutForWaitingOk, ctx: &mut Context<Self>) {
        let round = msg.round;

        ctx.run_later(Duration::from_millis(TIMEOUT_FOR_OK_MS), move |act, _| {
            if let ElectionStatus::WaitingOk { round: r } = act.election_status
                && r == round
            {
                act.leader = Some(act.id);
                act.round = round;
                act.election_status = ElectionStatus::Leader { round };
                act.cm.do_send(SetNewLeader { peer_id: act.id });

                for (&peer_id, &addr) in act.peers.iter() {
                    if peer_id == act.id {
                        continue;
                    }
                    let data = ProtocolMessage::ImLeader {
                        from: act.id,
                        round,
                    };
                    act.udp_io.do_send(SendUdp { data, target: addr });
                    trace!(
                        "[EM {}] Sending msg ImLeader to peer: {peer_id}, Round: {round}",
                        act.id
                    );
                }
            }
        });
    }
}
impl Handler<TimeoutForWaitingLeader> for ElectionManager {
    type Result = ();
    fn handle(&mut self, msg: TimeoutForWaitingLeader, ctx: &mut Context<Self>) {
        let round = msg.round;
        ctx.run_later(
            Duration::from_millis(TIMEOUT_FOR_LEADER_MS),
            move |act, ctx| {
                match act.election_status {
                    ElectionStatus::WaitingLeader { round: r } if r == round => {
                        let new_round = next_round();
                        trace!(
                            "[EM {}] Timeout waiting ImLeader. Restarting election with new round {} (old={})",
                            act.id,
                            new_round,
                            round
                        );
                        ctx.address().do_send(CallElection { round: new_round });
                    }
                    _ => {}
                };
            },
        );
    }
}
