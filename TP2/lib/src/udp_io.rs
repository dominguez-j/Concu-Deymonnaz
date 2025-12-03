use crate::messages::protocol::ProtocolMessage;
use crate::serde_cluster::udp::{deserialize_udp_msg, serialize_udp_msg};
use crate::utils::connection_socket_addr;
use crate::{elog, log};
use actix::{Actor, Addr, Context, Handler, Message, Recipient};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

const MAX_DATAGRAM: usize = 2048;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUdp {
    pub data: ProtocolMessage,
    pub target: SocketAddr,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UdpSubscribe {
    pub addr: Recipient<UdpInbound>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct UdpInbound {
    pub msg: ProtocolMessage,
    pub from: SocketAddr,
}
///* Maneja la recepción y envío de mensajes a través de una conexión UDP
///* Tiene una serie de subscriptores que implementan el msg "UdpInbound"
#[allow(dead_code)]
pub struct UdpIO {
    id: u32,
    socket: Arc<UdpSocket>,
    suscribers: Vec<Recipient<UdpInbound>>,
}

impl UdpIO {
    pub async fn start_new(id: u32, base_port: u32) -> Addr<Self> {
        let socket_addr = connection_socket_addr(base_port, id);
        let socket = Arc::new(
            UdpSocket::bind(socket_addr)
                .await
                .expect("internode_udp_addr"),
        );

        let udp_io_addr = UdpIO {
            id,
            socket: socket.clone(),
            suscribers: vec![],
        }
        .start();
        log!(
            "[UDPIO {}] Inbound UDP started on port: {}",
            id,
            socket_addr.port()
        );

        let udp_io_addr_clone = udp_io_addr.clone();
        actix::spawn(async move {
            loop {
                let mut buf = [0u8; MAX_DATAGRAM];
                match socket.recv_from(&mut buf).await {
                    Ok((size, from)) => {
                        let msg = deserialize_udp_msg(&buf, size);
                        udp_io_addr_clone.do_send(UdpInbound { msg, from });
                    }
                    Err(e) => elog!("[UDP {}] recv error: {}", id, e),
                }
            }
        });

        udp_io_addr
    }
}

impl Actor for UdpIO {
    type Context = Context<Self>;
}

impl Handler<UdpInbound> for UdpIO {
    type Result = ();
    /// Redirige los mensajes a los subscriptores que les corresponda
    fn handle(&mut self, msg: UdpInbound, _: &mut Context<Self>) {
        for recipient in &self.suscribers {
            recipient.do_send(msg.clone());
        }
    }
}

impl Handler<UdpSubscribe> for UdpIO {
    type Result = ();
    fn handle(&mut self, msg: UdpSubscribe, _: &mut Context<Self>) {
        self.suscribers.push(msg.addr);
    }
}

impl Handler<SendUdp> for UdpIO {
    type Result = ();
    ///Serializa y envía un mensaje al respectivo socket UDP
    fn handle(&mut self, msg: SendUdp, _: &mut Context<Self>) {
        let socket = self.socket.clone();
        let protocol_msg = msg.data;
        let serialized_msg = serialize_udp_msg(&protocol_msg);
        actix::spawn(async move {
            match socket.send_to(&serialized_msg, msg.target).await {
                Ok(_) => {}
                Err(e) => {
                    elog!("Failed to send UDP message in UdpIO. Error: {}", e);
                }
            }
        });
    }
}
