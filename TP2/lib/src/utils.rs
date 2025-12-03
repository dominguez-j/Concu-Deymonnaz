use crate::constants::general::HOST;
use std::net::SocketAddr;

///* Genera un socket addr a partir del Host utiizado, el puerto base correspondiente y el id de
///  la respectiva entidad
pub fn connection_socket_addr(base: u32, id: u32) -> SocketAddr {
    let port = base + id;
    let socket_addr: SocketAddr = format!("{}:{}", HOST, port)
        .parse()
        .expect("internode_conn_addr");
    socket_addr
}

pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

///Genera el nuevo "round" para la elección de líder
pub fn next_round() -> u64 {
    now_ms()
}
