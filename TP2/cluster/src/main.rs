use crate::managers::connection_manager::handlers::*;
use crate::managers::connection_manager::*;
use crate::managers::election_manager::*;
use crate::managers::heartbeat_manager::*;
use crate::managers::repository_manager::*;
use crate::managers::transaction_manager::*;
use crate::server_peer::*;
use actix::{Actor, Addr, StreamHandler};
use lib::cluster_tcp_writer::tcp_writer::TcpWriter;
use lib::config::Config;
use lib::constants::general::INTERNODE_BASE_PORT;
use lib::messages::protocol::ProtocolMessage;
use lib::roles::Role;
use lib::serde_cluster::tcp::serialize_tcp_msg;
use lib::udp_io::UdpIO;
use lib::utils::connection_socket_addr;
use lib::{elog, log};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::LinesStream;

mod locks;
mod managers;
mod messages;
mod repository;
mod server_peer;
mod spending_info;
mod transaction_with_id;

#[actix_rt::main]
async fn main() {
    let current_id = read_id_from_io();
    let cfg = Config::default();

    if let Err(e) = init_cluster(current_id, cfg.clone()).await {
        elog!("[INTERNODE_PEER {}] execution failed: {}", current_id, e);
    };

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
    elog!("shutdown");
}

fn read_id_from_io() -> u32 {
    std::env::args()
        .nth(1)
        .expect("usage: poc_cluster <id>")
        .parse()
        .expect("id must be u32")
}

async fn init_cluster(current_id: u32, cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let server_port_base = cfg.server_port_base;
    let num_nodes = cfg.num_nodes;

    let connection_manager = ConnectionManager::new(current_id, cfg.clone())
        .await
        .start();
    let repository_manager = RepositoryManager::new();
    let transaction_manager = TransactionManager::new(
        current_id,
        repository_manager.clone(),
        connection_manager.clone(),
    );
    repository_manager.do_send(SetTransactionManagerAddr {
        addr: transaction_manager.clone(),
    });

    let udp_io = UdpIO::start_new(current_id, server_port_base).await;

    let hb = HeartbeatManager::new(
        current_id,
        connection_manager.clone(),
        udp_io.clone(),
        cfg.clone(),
    )
    .start();

    let em = ElectionManager::new(current_id, udp_io.clone(), connection_manager.clone()).start();

    connection_manager.do_send(SetHeartbeatAddr { addr: hb.clone() });
    connection_manager.do_send(SetElectionManagerAddr { addr: em.clone() });
    connection_manager.do_send(SetTransactionManagerAddr {
        addr: transaction_manager.clone(),
    });
    if let Err(e) = init_connection_manager(
        connection_manager.clone(),
        transaction_manager.clone(),
        current_id,
        num_nodes,
    )
    .await
    {
        elog!("[INTERNODE_PEER {}] connect failed: {}", current_id, e);
    };

    Ok(())
}

async fn spawn_internode_dialer(
    current_id: u32,
    cm: Addr<ConnectionManager>,
    tm: Addr<TransactionManager>,
    num_nodes: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    actix::spawn(async move {
        if let Err(e) =
            create_internode_server_peer_from_connection_send(cm, tm, num_nodes, current_id, None)
                .await
        {
            elog!("dialer error: {e}");
        }
    });
    Ok(())
}
async fn init_connection_manager(
    connection_manager: Addr<ConnectionManager>,
    transaction_manager: Addr<TransactionManager>,
    current_id: u32,
    num_nodes: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    spawn_tcp_listener(
        current_id,
        connection_manager.clone(),
        transaction_manager.clone(),
        INTERNODE_BASE_PORT,
    )
    .await?;

    spawn_internode_dialer(
        current_id,
        connection_manager.clone(),
        transaction_manager.clone(),
        num_nodes,
    )
    .await?;

    Ok(())
}

async fn spawn_tcp_listener(
    current_id: u32,
    cm: Addr<ConnectionManager>,
    tm: Addr<TransactionManager>,
    base_port: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    actix::spawn(async move {
        let socket_addr: SocketAddr = connection_socket_addr(base_port, current_id);
        if let Err(e) =
            create_server_peer_from_connection_recv(cm, tm, current_id, socket_addr).await
        {
            elog!("listener error: {e} with id: {current_id}");
        }
    });
    Ok(())
}

///* Se instancia un actor de tipo ServerPeer genérico, que tendrá un role asignado a partir del
///  StartUp message asignando a dicho actor la parte de lectura del socket y en su estado interno
///  la parte de escritura
fn create_server_peer_for(
    tcp_stream: TcpStream,
    internode_id: u32,
    connection_manager: Addr<ConnectionManager>,
    transaction_manager: Addr<TransactionManager>,
    their_id_hint: Option<u32>,
) {
    let server_peer = ServerPeer::create(|ctx| {
        let (read_half, write_half) = split(tcp_stream);

        let write = Some(write_half);
        let writer = TcpWriter { write }.start();

        ServerPeer::add_stream(LinesStream::new(BufReader::new(read_half).lines()), ctx);
        ServerPeer::new(
            internode_id,
            writer,
            connection_manager.clone(),
            transaction_manager.clone(),
        )
    });
    // /Habrá un Hint siempre que se haga un Connect, es decir, sólo pasará con los internodes
    if let Some(their) = their_id_hint {
        connection_manager.do_send(RegisterPeer {
            id: their,
            address: server_peer.clone(),
            role: Role::Internode,
        });
    }
}

async fn create_server_peer_from_connection_recv(
    connection_manager: Addr<ConnectionManager>,
    transaction_manager: Addr<TransactionManager>,
    internode_id: u32,
    socket_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let tcp_listener = TcpListener::bind(socket_addr).await?;

    log!(
        "[CLUSTER] Server started on internode port {}",
        socket_addr.port()
    );

    loop {
        match tcp_listener.accept().await {
            Ok((stream, _peer_addr)) => {
                log!("[CLUSTER] id {}: TCP accept", internode_id);
                create_server_peer_for(
                    stream,
                    internode_id,
                    connection_manager.clone(),
                    transaction_manager.clone(),
                    None,
                );
            }
            Err(e) => elog!("[CLUSTER] id {}. Accept error: {}", internode_id, e),
        }
    }
}

async fn create_internode_server_peer_from_connection_send(
    connection_manager: Addr<ConnectionManager>,
    transaction_manager: Addr<TransactionManager>,
    num_nodes: u32,
    current_id: u32,
    peers_to_connect: Option<Vec<u32>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let peers = match peers_to_connect {
        Some(peers) => {
            log!(
                "[CLUSTER] id {}: preparing to connect to peer {:?}",
                current_id,
                peers
            );
            peers
        }
        None => (1..=num_nodes)
            .filter(|&id| id < current_id)
            .collect::<Vec<_>>(),
    };

    for &peer_id in &peers {
        let socket_addr = connection_socket_addr(INTERNODE_BASE_PORT, peer_id);
        match TcpStream::connect(socket_addr).await {
            Ok(mut stream) => {
                log!(
                    "[DIAL {}] connected to peer_id = {} at {}",
                    current_id,
                    peer_id,
                    socket_addr
                );

                //Envío del current_id para Hand Shake y que sea registrado por los otros peers
                let init_id_msg = ProtocolMessage::StartUp {
                    from: current_id,
                    role: Role::Internode,
                };
                let line = serialize_tcp_msg(&init_id_msg);
                if let Err(e) = stream.write_all(line.as_bytes()).await {
                    eprintln!("[DIAL {}] Can't send StartUp msg: {e}", current_id);
                    continue;
                }
                create_server_peer_for(
                    stream,
                    current_id,
                    connection_manager.clone(),
                    transaction_manager.clone(),
                    Some(peer_id),
                );
            }
            Err(e) => {
                elog!(
                    "[DIAL {}] connect {} failed: {}",
                    current_id,
                    socket_addr,
                    e
                );
                continue;
            }
        };
    }

    Ok(())
}
