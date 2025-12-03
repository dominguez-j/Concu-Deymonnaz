use crate::ipc::tcp_writer_actor::TcpWriterActor;
use actix::prelude::*;
use std::ops::{Deref, DerefMut};
use tokio::net::tcp::OwnedWriteHalf;

pub struct TcpWriter {
    actor: Addr<TcpWriterActor>,
}

impl TcpWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        let actor = TcpWriterActor::new(writer).start();
        Self { actor }
    }
}

impl Deref for TcpWriter {
    type Target = Addr<TcpWriterActor>;
    fn deref(&self) -> &Self::Target {
        &self.actor
    }
}

impl DerefMut for TcpWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::representable::Representable;
    use tokio::{io::AsyncReadExt, net::TcpListener};

    #[derive(Debug, PartialEq, Message, serde::Serialize, serde::Deserialize)]
    #[rtype(result = "()")]
    struct TestMessage(pub String);

    impl Representable for TestMessage {}

    #[actix::test]
    async fn test_tcp_writer_actor_sends_data_correctly() {
        let listener = TcpListener::bind("127.0.0.1:6002").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (_, write_half) = client.into_split();

        let actor = TcpWriter::new(write_half);
        actor.do_send(TestMessage("Hola".to_string()));

        let mut buffer = vec![0u8; 64];
        let n = server_socket.read(&mut buffer).await.unwrap();
        let received = String::from_utf8_lossy(&buffer[..n]).to_string();

        assert_eq!(
            TestMessage::from_representation(received),
            TestMessage("Hola".to_string())
        );
    }
}
