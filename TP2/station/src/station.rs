use crate::connection_establisher::ConnectionEstablisher;
use crate::heartbeat_sender::HeartbeatSender;
use crate::message::payments_map::SavedPayments;
use crate::message::pump_connected::PumpConnected;
use crate::message::{
    clear::Clear, connection_established::ConnectionEstablished, get_all::GetAll,
    register_heartbeat::RegisterHeartbeat, remove::Remove,
};
use crate::pump_connection_listener::PumpConnectionListener;
use crate::repository_manager::RepositoryManager;
use actix::prelude::*;
use lib::prelude::*;
use std::collections::HashMap;
use tokio::net::TcpStream;

pub struct Station {
    id: u32,
    counter: u32,
    heartbeat_manager: Addr<HeartbeatManager<Self>>,
    connection_establisher: Addr<ConnectionEstablisher>,
    _pump_connection_listener: PumpConnectionListener,
    heartbeat_sender: Addr<HeartbeatSender>,
    repository_manager: Addr<RepositoryManager>,
    pumps_protocols: HashMap<u32, TcpProtocol<Self, Payment>>,
    node_protocol: Option<TcpProtocol<Self, ProtocolMessage>>,
}

impl Station {
    pub async fn new(
        id: u32,
        repository_name: String,
        ping_receive_port: u32,
        ping_send_base_port: u32,
        node_address: String,
        hostname: String,
        base_port: u32,
    ) -> Addr<Self> {
        Self::create(|ctx| {
            let connection_establisher =
                ConnectionEstablisher::new(ctx.address(), node_address.clone());
            connection_establisher.do_send(Deploy);
            let heartbeat_manager = HeartbeatManager::new(ctx.address(), id, ping_receive_port);
            let station = Self {
                id,
                counter: 0,
                heartbeat_manager,
                connection_establisher,
                _pump_connection_listener: PumpConnectionListener::new(
                    ctx.address(),
                    id,
                    hostname.clone(),
                    base_port,
                ),
                heartbeat_sender: HeartbeatSender::new(id, ping_send_base_port),
                repository_manager: RepositoryManager::new(ctx.address(), repository_name),
                pumps_protocols: HashMap::new(),
                node_protocol: None,
            };
            station
        })
    }
    fn connect_to_node(&mut self, socket: TcpStream, ctx: &mut Context<Self>) {
        let mut protocol = TcpProtocol::new(ctx.address(), socket);
        protocol.send(ProtocolMessage::StartUp {
            from: self.id,
            role: Role::Station,
        });
        self.node_protocol = Some(protocol);
    }
}

impl Actor for Station {
    type Context = Context<Self>;
}

impl Handler<Payment> for Station {
    type Result = ();
    fn handle(&mut self, mut msg: Payment, ctx: &mut Context<Self>) -> Self::Result {
        msg.set_station_id(self.id);
        log!("[STATION] Counter: {}]", self.counter);
        msg.set_id(format!(
            "{},{}",
            IdInterpreter::build_payment_id(
                msg.enterprise_id(),
                msg.card_id(),
                msg.pump_id().unwrap(),
                self.id,
            ),
            self.counter
        ));
        self.counter += 1;
        self.repository_manager.do_send(msg.clone());
        if let Some(protocol) = self.node_protocol.as_mut() {
            println!("Sending transaction");
            protocol.send(ProtocolMessage::Update(Update::Payment {
                payment_id: msg.id(),
                enterprise_id: msg.enterprise_id(),
                card_id: msg.card_id(),
                transaction_type: PaymentType::PaymentVerification,
                cost: msg.cost(),
            }))
        } else {
            println!("Sending accepted transaction on offline situation");
            ctx.address()
                .do_send(TransactionResult::new(msg.id(), true));
        }
    }
}

impl Handler<ProtocolMessage> for Station {
    type Result = ();
    fn handle(&mut self, msg: ProtocolMessage, ctx: &mut Context<Self>) {
        match msg {
            ProtocolMessage::TransactionResponse(
                TransactionResponse::TransactionResultResponse { result, .. },
            ) => {
                ctx.address().do_send(result);
            }
            ProtocolMessage::FirstMsg {
                from: id,
                data: msg,
            } => println!("From internode {} - data: {}", id, msg),
            _ => println!("unknown protocol message: {:?}", msg),
        }
    }
}

impl Handler<TransactionResult> for Station {
    type Result = ();
    fn handle(&mut self, msg: TransactionResult, _: &mut Context<Self>) -> Self::Result {
        let tr_id = msg.transaction_id().splitn(2, '+').collect::<Vec<&str>>()[0].to_string();
        let pump_id = IdInterpreter::get_pump_id(tr_id.to_string()).unwrap();
        if let Some(pump) = self.pumps_protocols.get_mut(&pump_id) {
            pump.send(PumpReceiveMessageType::TransactionResult(msg));
        }
        if self.node_protocol.is_some() {
            self.repository_manager
                .do_send(Remove::new(tr_id.to_string()));
        }
    }
}

impl Handler<ConnectionLost> for Station {
    type Result = ();
    fn handle(&mut self, _: ConnectionLost, _: &mut Context<Self>) -> Self::Result {
        self.node_protocol = None;
        self.repository_manager.do_send(GetAll);
        self.connection_establisher.do_send(Deploy);
    }
}

impl Handler<ConnectionEstablished> for Station {
    type Result = ();
    fn handle(&mut self, msg: ConnectionEstablished, ctx: &mut Context<Self>) -> Self::Result {
        self.connect_to_node(msg.socket(), ctx);
        self.heartbeat_manager.do_send(Deploy);
        self.repository_manager.do_send(GetAll);
        self.repository_manager.do_send(Clear);
    }
}

impl Handler<PumpConnected> for Station {
    type Result = ();
    fn handle(&mut self, msg: PumpConnected, ctx: &mut Context<Self>) -> Self::Result {
        let id = (self.pumps_protocols.len() + 1) as u32;
        let socket = msg.socket();
        let address = socket.peer_addr().unwrap();
        let mut protocol = TcpProtocol::new(ctx.address(), socket);
        protocol.send(PumpReceiveMessageType::SetPumpId(SetPumpId::new(id)));
        self.pumps_protocols.insert(id, protocol);
        self.heartbeat_sender
            .do_send(RegisterHeartbeat::new(id, address));
    }
}

impl Handler<SavedPayments> for Station {
    type Result = ();
    fn handle(&mut self, msg: SavedPayments, _: &mut Context<Self>) -> Self::Result {
        if let Some(protocol) = &mut self.node_protocol {
            msg.payments().iter().for_each(|payment| {
                protocol.send(ProtocolMessage::Update(Update::Payment {
                    payment_id: payment.id(),
                    enterprise_id: payment.enterprise_id(),
                    card_id: payment.card_id(),
                    transaction_type: PaymentType::ForcePayment,
                    cost: payment.cost(),
                }));
            });
        } else {
            msg.payments().iter().for_each(|payment| {
                let pump_id = payment.pump_id().unwrap();
                self.pumps_protocols.get_mut(&pump_id).unwrap().send(
                    PumpReceiveMessageType::TransactionResult(TransactionResult::new(
                        payment.id(),
                        true,
                    )),
                );
            });
        }
    }
}
