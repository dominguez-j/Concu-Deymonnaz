use actix::prelude::*;
use tokio::net::TcpStream;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectionEstablished {
    socket: TcpStream,
}

impl ConnectionEstablished {
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }
    pub fn socket(self) -> TcpStream {
        self.socket
    }
}
