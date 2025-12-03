use crate::enterprise::Enterprise;
use crate::messages::NewConnection;
use actix::prelude::*;
use lib::prelude::*;
use tokio::net::TcpStream;

pub struct Proxy {
    enterprise_id: u32,
    enterprise_addr: Addr<Enterprise>,
    heartbeat_manager: Option<Addr<HeartbeatManager<Self>>>,
    protocol: Option<TcpProtocol<Enterprise, ProtocolMessage>>,
    host: String,
    proxy_port: u32,
    internode_port: u32,
    node_count: u32,
}

impl Actor for Proxy {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat_manager = Some(HeartbeatManager::new(
            ctx.address(),
            self.enterprise_id,
            self.proxy_port,
        ));

        if let Some(heartbeat_manager) = &self.heartbeat_manager {
            heartbeat_manager.do_send(Deploy);
        }
    }
}

impl Proxy {
    pub async fn new(
        enterprise_addr: Addr<Enterprise>,
        enterprise_id: u32,
        host: String,
        proxy_port: u32,
        internode_port: u32,
        node_count: u32,
    ) -> Addr<Self> {
        let tcp_stream =
            Self::connect_to_any_node(host.clone(), internode_port.clone(), node_count.clone())
                .await
                .unwrap();
        Self::create(move |_ctx| {
            let protocol = Some(TcpProtocol::new(enterprise_addr.clone(), tcp_stream));

            let proxy = Self {
                enterprise_id,
                enterprise_addr,
                heartbeat_manager: None,
                protocol,
                host,
                proxy_port,
                internode_port,
                node_count,
            };

            proxy
        })
    }

    pub async fn connect_to_any_node(
        host: String,
        internode_port: u32,
        node_count: u32,
    ) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let mut i = 1;
        loop {
            let addr = format!("{}:{}", host, internode_port + i);

            match TcpStream::connect(&addr).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    eprintln!("Failed to connect to node {}: {}. Retrying...", i, e);
                }
            }

            i += 1;
            if i > node_count {
                i = 1;
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

impl Handler<ProtocolMessage> for Proxy {
    type Result = ();

    fn handle(&mut self, msg: ProtocolMessage, _ctx: &mut Context<Self>) {
        self.protocol.as_mut().unwrap().send(msg);
    }
}

impl Handler<ConnectionLost> for Proxy {
    type Result = ();

    fn handle(&mut self, _msg: ConnectionLost, ctx: &mut Context<Self>) {
        self.protocol = None;
        let proxy_addr = ctx.address();
        let host = self.host.clone();
        let internode_port = self.internode_port.clone();
        let node_count = self.node_count.clone();

        ctx.spawn(
            async move {
                match Proxy::connect_to_any_node(host, internode_port, node_count).await {
                    Ok(tcp_stream) => {
                        proxy_addr.do_send(NewConnection(tcp_stream));
                    }
                    Err(e) => {
                        eprintln!("Reconnection error: {}", e);
                    }
                }
            }
            .into_actor(self),
        );
    }
}

impl Handler<NewConnection> for Proxy {
    type Result = ();

    fn handle(&mut self, msg: NewConnection, _: &mut Context<Self>) {
        let tcp_stream = msg.0;
        self.protocol = Some(TcpProtocol::new(self.enterprise_addr.clone(), tcp_stream));
        if let Some(heartbeat_manager) = &self.heartbeat_manager {
            heartbeat_manager.do_send(Deploy);
        }
        self.protocol
            .as_mut()
            .unwrap()
            .send(ProtocolMessage::StartUp {
                from: self.enterprise_id,
                role: Role::Proxy,
            });
    }
}
