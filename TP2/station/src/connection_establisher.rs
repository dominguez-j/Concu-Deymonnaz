use crate::message::connection_established::ConnectionEstablished;
use crate::station::Station;
use actix::prelude::*;
use lib::prelude::*;
use std::time::Duration;
use tokio::net::TcpStream;

pub struct ConnectionEstablisher {
    station: Addr<Station>,
    socket_addr: String,
    connect_interval: Option<SpawnHandle>,
}

impl ConnectionEstablisher {
    pub fn new(station: Addr<Station>, socket_addr: String) -> Addr<Self> {
        Self {
            station,
            socket_addr,
            connect_interval: None,
        }
        .start()
    }
    fn build_connect_interval(&mut self, ctx: &mut <Self as Actor>::Context) {
        let socket_addr = self.socket_addr.clone();
        self.connect_interval = Some(ctx.run_interval(
            Duration::from_secs(5),
            move |connection_stablisher, ctx| {
                let socket_addr_copy = socket_addr.clone();
                let socket_addr_copy_2 = socket_addr.clone();
                println!("Trying to connect to {}", socket_addr_copy);
                let connection_future = async move { TcpStream::connect(socket_addr_copy).await }
                    .into_actor(connection_stablisher)
                    .map(move |result, _, ctx| match result {
                        Ok(stream) => {
                            ctx.address().do_send(ConnectionEstablished::new(stream));
                            println!("Connected to {}", socket_addr_copy_2);
                        }
                        Err(_) => {
                            println!("Failed to connect to {}", socket_addr_copy_2);
                        }
                    });
                ctx.spawn(connection_future);
            },
        ));
    }
}

impl Actor for ConnectionEstablisher {
    type Context = Context<Self>;
}

impl Handler<Deploy> for ConnectionEstablisher {
    type Result = ();
    fn handle(&mut self, _msg: Deploy, ctx: &mut Context<Self>) {
        self.build_connect_interval(ctx);
    }
}

impl Handler<ConnectionEstablished> for ConnectionEstablisher {
    type Result = ();
    fn handle(&mut self, msg: ConnectionEstablished, ctx: &mut Context<Self>) {
        println!("Cancelling connection process");
        ctx.cancel_future(self.connect_interval.take().unwrap());
        self.station.do_send(msg);
    }
}
