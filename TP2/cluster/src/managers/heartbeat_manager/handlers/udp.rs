use crate::HeartbeatManager;
use crate::managers::connection_manager::handlers::NewPeerDetectedFromHB;
use actix::{Context, Handler};
use lib::messages::protocol::ProtocolMessage;
use lib::prelude::*;
use lib::udp_io::UdpInbound;
use std::time::Instant;

impl Handler<UdpInbound> for HeartbeatManager {
    type Result = ();

    fn handle(&mut self, msg: UdpInbound, _: &mut Context<Self>) {
        match msg.msg {
            ProtocolMessage::HbMsg { from, .. } if from != self.id => {
                if let Some(st) = self.peers.get_mut(&from) {
                    // trace!(
                    //     "[HB {}] PING received from: {} - actual misses: {:?}. Delay: {}",
                    //     self.id,
                    //     &from,
                    //     st.misses,
                    //     st.last_seen.elapsed().as_millis()
                    // );
                    st.misses = 0;
                    st.last_seen = Instant::now();
                } else {
                    self.cm.do_send(NewPeerDetectedFromHB { peer_id: from });
                }
            }
            _ => {}
        }
    }
}
