use crate::message::register_heartbeat::RegisterHeartbeat;
use actix::prelude::*;
use lib::prelude::*;
use std::net::SocketAddr;
use std::time::Duration;

pub struct HeartbeatSender {
    id: u32,
    base_port: u32,
    udp_io: Option<Addr<UdpIO>>,
    addresses: Vec<SocketAddr>,
}

impl HeartbeatSender {
    pub fn new(id: u32, base_port: u32) -> Addr<Self> {
        Self::create(|ctx| {
            let sender = Self {
                id,
                base_port,
                udp_io: None,
                addresses: vec![],
            };
            ctx.wait(
                async move { UdpIO::start_new(0, base_port).await }
                    .into_actor(&sender)
                    .map(|udp, s, _| {
                        s.udp_io = Some(udp);
                    }),
            );
            ctx.run_interval(Duration::from_secs(1), |sender, _| {
                if let Some(udp) = &sender.udp_io {
                    sender.addresses.iter_mut().for_each(|address| {
                        udp.do_send(SendUdp {
                            data: ProtocolMessage::HbMsg {
                                from: sender.id,
                                ts_ms: now_ms(),
                            },
                            target: *address,
                        });
                    });
                }
            });
            sender
        })
    }
}

impl Actor for HeartbeatSender {
    type Context = Context<Self>;
}

impl Handler<RegisterHeartbeat> for HeartbeatSender {
    type Result = ();
    fn handle(&mut self, msg: RegisterHeartbeat, _: &mut Context<Self>) {
        let id = msg.id();
        let mut address = msg.address();
        address.set_port((self.base_port + id) as u16);
        println!("Added a send to port {}", address.port());
        self.addresses.push(address);
    }
}
