use crate::message::pump_connected::PumpConnected;
use crate::station::Station;
use actix::prelude::*;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct PumpConnectionListener {
    _task: JoinHandle<()>,
}

impl PumpConnectionListener {
    pub fn new(station: Addr<Station>, id: u32, hostname: String, base_port: u32) -> Self {
        Self {
            _task: Self::listen(station, id, hostname, base_port),
        }
    }
    fn listen(station: Addr<Station>, id: u32, hostname: String, base_port: u32) -> JoinHandle<()> {
        actix::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{}", hostname, base_port + id))
                .await
                .unwrap();
            loop {
                if let Ok((incoming, _)) = listener.accept().await {
                    station.do_send(PumpConnected::new(incoming));
                }
            }
        })
    }
}
