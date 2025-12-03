use actix::prelude::*;
use std::net::SocketAddr;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterHeartbeat {
    id: u32,
    address: SocketAddr,
}

impl RegisterHeartbeat {
    pub fn new(id: u32, address: SocketAddr) -> Self {
        Self { id, address }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}
