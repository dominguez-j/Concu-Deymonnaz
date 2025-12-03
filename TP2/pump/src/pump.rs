use crate::connection_establisher::ConnectionEstablisher;
use actix::prelude::*;
use lib::prelude::*;
use lib::pump_receive_type::PumpReceiveMessageType;
use std::collections::VecDeque;

pub struct Pump {
    id: Option<u32>,
    ping_receive_port: u32,
    connection_establisher: Option<ConnectionEstablisher>,
    heartbeat_manager: Option<Addr<HeartbeatManager<Self>>>,
    protocol: Option<TcpProtocol<Self, PumpReceiveMessageType>>,
    pending: VecDeque<Payment>,
}

impl Pump {
    pub fn new(station_address: String, ping_receive_port: u32) -> Addr<Self> {
        Self::create(|_| {
            let pump = Self {
                id: None,
                ping_receive_port,
                connection_establisher: Some(ConnectionEstablisher::new(station_address)),
                heartbeat_manager: None,
                protocol: None,
                pending: VecDeque::new(),
            };
            pump
        })
    }
    fn connect_to_station(&mut self, ctx: &mut <Pump as Actor>::Context) {
        let conn_establisher = std::mem::take(&mut self.connection_establisher).unwrap();
        ctx.wait(
            async move {
                (
                    conn_establisher.generate_connection().await,
                    conn_establisher,
                )
            }
            .into_actor(self)
            .map(|(socket, conn_establisher), pump, ctx| {
                pump.connection_establisher = Some(conn_establisher);
                pump.protocol = Some(TcpProtocol::new(ctx.address(), socket));
                if let Some(hmanager) = pump.heartbeat_manager.as_mut() {
                    hmanager.do_send(Deploy);
                }
            }),
        );
    }
}

impl Actor for Pump {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.connect_to_station(ctx);
        println!("Pump started! Still waiting for id");
    }
}

impl Handler<ConnectionLost> for Pump {
    type Result = ();
    fn handle(&mut self, _msg: ConnectionLost, ctx: &mut Self::Context) {
        self.protocol = None;
        self.connect_to_station(ctx);
    }
}

impl Handler<PumpReceiveMessageType> for Pump {
    type Result = ();
    fn handle(&mut self, msg: PumpReceiveMessageType, ctx: &mut Self::Context) {
        match msg {
            PumpReceiveMessageType::SetPumpId(pump_id) => ctx.address().do_send(pump_id),
            PumpReceiveMessageType::TransactionResult(transaction_result) => {
                ctx.address().do_send(transaction_result)
            }
        }
    }
}

impl Handler<Payment> for Pump {
    type Result = ();
    fn handle(&mut self, mut msg: Payment, _: &mut Context<Self>) -> Self::Result {
        if let Some(id) = self.id {
            println!("Sending payment");
            msg.set_pump_id(id);
            self.protocol.as_mut().unwrap().send(msg);
        } else {
            self.pending.push_back(msg);
        }
    }
}

impl Handler<SetPumpId> for Pump {
    type Result = ();
    fn handle(&mut self, msg: SetPumpId, ctx: &mut Self::Context) {
        self.id = Some(msg.id());
        if self.heartbeat_manager.is_none() {
            self.heartbeat_manager = Some(HeartbeatManager::new(
                ctx.address(),
                self.id.unwrap(),
                self.ping_receive_port,
            ));
        }
        self.heartbeat_manager.as_mut().unwrap().do_send(Deploy);
        if self.pending.len() > 0 {
            ctx.address().do_send(self.pending.pop_back().unwrap());
        }
    }
}

impl Handler<TransactionResult> for Pump {
    type Result = ();
    fn handle(&mut self, msg: TransactionResult, _: &mut Context<Self>) -> Self::Result {
        println!("{}", msg.as_representation());
    }
}
