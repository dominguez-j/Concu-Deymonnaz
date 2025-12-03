use crate::ConnectionManager;
use crate::locks::*;
use crate::managers::connection_manager::handlers::RegisterPeer;
use crate::managers::lock_manager::Pending;
use crate::managers::transaction_manager::{Transaction, TransactionManager};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler, prelude::*};
use lib::cluster_tcp_writer::tcp_writer::{ShutdownTcpWriter, TcpWriter, WriteMessage};
use lib::messages::response::InternodeResponse;
use lib::messages::set_own_data::SendBroadcastOfSetOwnData;
use lib::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ShutdownServer;

#[derive(Message)]
#[rtype(result = "()")]
pub struct TransactionResponseFromInternode {
    pub(crate) response: InternodeResponse,
    pub(crate) original_id: String,
    pub(crate) from: Addr<ServerPeer>,
    pub(crate) initial_leader: Option<u32>,
}

pub struct ServerPeer {
    pub(crate) id: u32,
    pub(crate) role: Option<Role>,
    pub(crate) writer: Addr<TcpWriter>,
    pub(crate) connection_manager: Addr<ConnectionManager>,
    pub(crate) transaction_manager: Addr<TransactionManager>,
}

impl ServerPeer {
    pub fn new(
        id: u32,
        writer: Addr<TcpWriter>,
        connection_manager: Addr<ConnectionManager>,
        transaction_manager: Addr<TransactionManager>,
    ) -> Self {
        Self {
            id,
            role: None,
            writer,
            connection_manager,
            transaction_manager,
        }
    }

    fn send_protocol_message(&self, msg: ProtocolMessage) {
        self.writer.do_send(WriteMessage {
            line: serialize_tcp_msg(&msg),
        });
    }
}

impl Actor for ServerPeer {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for ServerPeer {
    fn handle(&mut self, line: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        let buf = match line {
            Ok(msg) => msg,
            Err(e) => {
                elog!("[INTERNODE_PEER {}] read error: {}", self.id, e);
                return;
            }
        };

        let msg = deserialize_tcp_msg(&buf);

        match msg {
            ProtocolMessage::StartUp {
                from: peer_id,
                role,
            } => {
                self.role = Some(role);
                self.connection_manager.do_send(RegisterPeer {
                    id: peer_id,
                    address: ctx.address(),
                    role: self.role.clone().unwrap(),
                });
                let data = format!(
                    "Hello from Server Peer {}, StartUp msg from {:?}: {peer_id} received ",
                    self.id,
                    self.role.clone().unwrap()
                );
                let first_msg = ProtocolMessage::FirstMsg {
                    from: self.id,
                    data,
                };
                self.send_protocol_message(first_msg);
            }
            ProtocolMessage::FirstMsg {
                from: peer_id,
                data: msg,
            } => {
                if let Some(role) = self.role.clone() {
                    trace!(
                        "[INTERNODE {}] First Message received from {role:?} {peer_id}. '\n'Message: {msg:?}",
                        self.id
                    );
                }
            }
            ProtocolMessage::Update(update) => self.transaction_manager.do_send(Transaction {
                owner: ctx.address(),
                id: None,
                transaction: TransactionType::Update(update),
                initial_leader: None,
            }),
            ProtocolMessage::Select(select) => self.transaction_manager.do_send(Transaction {
                owner: ctx.address(),
                id: None,
                transaction: TransactionType::Select(select),
                initial_leader: None,
            }),
            ProtocolMessage::EnterpriseCreate {
                enterprise_id,
                enterprise_balance: enterprise_limit,
                card_limits,
            } => self.transaction_manager.do_send(Transaction {
                owner: ctx.address(),
                id: None,
                transaction: TransactionType::Create(Create {
                    enterprise_id,
                    enterprise_limit,
                    card_limits,
                }),
                initial_leader: None,
            }),
            ProtocolMessage::RequestData(request) => {
                self.transaction_manager.do_send(Transaction {
                    owner: ctx.address(),
                    id: None,
                    transaction: TransactionType::RequestData(request.clone()),
                    initial_leader: request.initial_leader,
                });
            }
            ProtocolMessage::InternodeResponse(response) => {
                self.transaction_manager
                    .do_send(TransactionResponseFromInternode {
                        from: ctx.address(),
                        original_id: response.get_original_transaction_id().clone(),
                        initial_leader: response.get_initial_leader(),
                        response,
                    })
            }
            ProtocolMessage::InternodeCreate(create) => {
                self.transaction_manager.do_send(create);
            }
            ProtocolMessage::AcquireLock {
                transaction_id,
                enterprise_id,
                from,
            } => {
                self.transaction_manager.do_send(Pending {
                    owner: Some(ctx.address()),
                    owner_id: from,
                    transaction_id,
                    enterprise_id,
                    got_lock: false,
                });
            }
            ProtocolMessage::ReleaseLock {
                transaction_id,
                enterprise_id,
            } => {
                self.transaction_manager.do_send(ReleaseLock {
                    transaction_id,
                    enterprise_id,
                });
            }
            ProtocolMessage::LockGranted {
                transaction_id,
                enterprise_id,
            } => self.transaction_manager.do_send(LockGranted {
                transaction_id,
                enterprise_id,
            }),
            ProtocolMessage::InternodeSetOwnData(set_own_data) => {
                self.transaction_manager.do_send(set_own_data);
            }
            _other => {
                // Acá tendremos: QueryReq/Res y demás mensajes
                log!("[INTERNODE {}] Unexpected message: {:?}", self.id, _other);
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        log!(
            "[SERVER_PEER {}] Peer with Role: {:?}, stream finished",
            self.id,
            self.role
        );
    }
}

impl Handler<ShutdownServer> for ServerPeer {
    type Result = ();
    fn handle(&mut self, _msg: ShutdownServer, ctx: &mut Self::Context) {
        self.writer.do_send(ShutdownTcpWriter);
        ctx.stop();
    }
}

impl Handler<SendBroadcastOfSetOwnData> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: SendBroadcastOfSetOwnData, _ctx: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::InternodeSetOwnData(msg.command));
    }
}

impl Handler<AcquireLock> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: AcquireLock, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::AcquireLock {
            from: msg.from,
            transaction_id: msg.transaction_id,
            enterprise_id: msg.enterprise_id,
        });
    }
}

impl Handler<ReleaseLock> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: ReleaseLock, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::ReleaseLock {
            transaction_id: msg.transaction_id,
            enterprise_id: msg.enterprise_id,
        });
    }
}

impl Handler<LockGranted> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: LockGranted, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::LockGranted {
            transaction_id: msg.transaction_id,
            enterprise_id: msg.enterprise_id,
        });
    }
}

impl Handler<Create> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: Create, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::InternodeCreate(msg));
    }
}

impl Handler<TransactionResponse> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: TransactionResponse, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::TransactionResponse(msg));
    }
}

impl Handler<InternodeResponse> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: InternodeResponse, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::InternodeResponse(msg));
    }
}

impl Handler<RequestData> for ServerPeer {
    type Result = ();
    fn handle(&mut self, msg: RequestData, _: &mut Self::Context) {
        self.send_protocol_message(ProtocolMessage::RequestData(msg));
    }
}
