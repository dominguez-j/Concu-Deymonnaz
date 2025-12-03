use crate::heartbeat::connection_lost::ConnectionLost;
use crate::heartbeat::deploy::Deploy;
use crate::udp_io::{UdpIO, UdpInbound, UdpSubscribe};
use actix::dev::ToEnvelope;
use actix::prelude::*;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(3);

pub struct HeartbeatManager<T>
where
    T: Actor,
    T::Context: ToEnvelope<T, ConnectionLost>,
    T: Handler<ConnectionLost>,
{
    id: u32,
    port: u32,
    udp_io: Option<Addr<UdpIO>>,
    recipient: Addr<T>,
    ping_receive_interval: Option<SpawnHandle>,
    last_ping_time: Instant,
}

impl<T> HeartbeatManager<T>
where
    T: Actor,
    T::Context: ToEnvelope<T, ConnectionLost>,
    T: Handler<ConnectionLost>,
{
    pub fn new(recipient: Addr<T>, id: u32, port: u32) -> Addr<Self> {
        Self {
            id,
            port,
            udp_io: None,
            recipient,
            ping_receive_interval: None,
            last_ping_time: Instant::now(),
        }
        .start()
    }
    fn build_ping_receive_interval(&mut self, ctx: &mut Context<Self>) {
        if let Some(h) = self.ping_receive_interval.take() {
            ctx.cancel_future(h);
        }

        let recipient = self.recipient.clone();

        let handle = ctx.run_interval(TIMEOUT, move |actor, ctx| {
            let elapsed = actor.last_ping_time.elapsed();
            if elapsed > TIMEOUT {
                recipient.do_send(ConnectionLost);
                //println!("---- Receiving ping ----");
                if let Some(h) = actor.ping_receive_interval.take() {
                    ctx.cancel_future(h);
                }
            }
        });

        self.ping_receive_interval = Some(handle);
    }
}

impl<T> Actor for HeartbeatManager<T>
where
    T: Actor,
    T::Context: ToEnvelope<T, ConnectionLost>,
    T: Handler<ConnectionLost>,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let id = self.id;
        let port = self.port;
        ctx.wait(
            async move {
                let udp_io = UdpIO::start_new(id, port).await;
                udp_io
            }
            .into_actor(self)
            .map(|udp_io, ping_receiver, ctx| {
                udp_io.do_send(UdpSubscribe {
                    addr: ctx.address().recipient(),
                });
                ping_receiver.udp_io = Some(udp_io);
            }),
        );
    }
}

impl<T> Handler<UdpInbound> for HeartbeatManager<T>
where
    T: Actor,
    T::Context: ToEnvelope<T, ConnectionLost>,
    T: Handler<ConnectionLost>,
{
    type Result = ();
    fn handle(&mut self, _msg: UdpInbound, _: &mut Context<Self>) -> Self::Result {
        //println!("Ping received from {} - Message: {:?}", msg.from, msg.msg);
        self.last_ping_time = Instant::now();
    }
}

impl<T> Handler<Deploy> for HeartbeatManager<T>
where
    T: Actor,
    T: Handler<ConnectionLost>,
    T::Context: ToEnvelope<T, ConnectionLost>,
{
    type Result = ();
    fn handle(&mut self, _: Deploy, ctx: &mut Context<Self>) {
        self.last_ping_time = Instant::now();
        self.build_ping_receive_interval(ctx);
    }
}
