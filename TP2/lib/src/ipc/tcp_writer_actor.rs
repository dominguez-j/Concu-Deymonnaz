use crate::ipc::representable::Representable;
use actix::prelude::*;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::mpsc, task};

const CHANNEL_SIZE: usize = 128;

pub struct TcpWriterActor {
    sender_channel: mpsc::Sender<String>,
}

impl TcpWriterActor {
    pub fn new(mut writer: OwnedWriteHalf) -> Self {
        let (tx, mut rx) = mpsc::channel::<String>(CHANNEL_SIZE);
        task::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(_) = writer.write_all(&data.into_bytes()).await {
                    break;
                }
                if let Err(_) = writer.flush().await {
                    break;
                }
            }
        });
        Self { sender_channel: tx }
    }
}

impl Actor for TcpWriterActor {
    type Context = Context<Self>;
}

impl<T> Handler<T> for TcpWriterActor
where
    T: Representable + Message<Result = ()>,
{
    type Result = ();
    fn handle(&mut self, msg: T, _ctx: &mut Self::Context) -> Self::Result {
        let data = msg.as_representation();
        let tx = self.sender_channel.clone();
        actix::spawn(async move { if let Err(_) = tx.send(data + "\n").await {} });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::{io::AsyncReadExt, net::TcpListener};

    #[derive(Debug, Clone, PartialEq, Message, Serialize, Deserialize)]
    #[rtype(result = "()")]
    struct TestMessage(pub String);

    impl Representable for TestMessage {}

    #[actix::test]
    async fn test_tcp_writer_actor_sends_data_correctly() {
        let listener = TcpListener::bind("127.0.0.1:6000").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (_, write_half) = client.into_split();

        let actor = TcpWriterActor::new(write_half).start();
        actor.do_send(TestMessage("Hola".to_string()));

        let mut buffer = vec![0u8; 64];
        let n = server_socket.read(&mut buffer).await.unwrap();
        let received = String::from_utf8_lossy(&buffer[..n]).to_string();

        assert_eq!(
            TestMessage::from_representation(received[..received.len() - 1].to_string()),
            TestMessage("Hola".to_string())
        );
    }

    #[actix::test]
    async fn test_tcp_writer_actor_sends_multiple_messages_correctly() {
        let listener = TcpListener::bind("127.0.0.1:6001").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (_, write_half) = client.into_split();

        let actor = TcpWriterActor::new(write_half).start();

        let messages = vec![
            TestMessage("Hola".into()),
            TestMessage("Mundo".into()),
            TestMessage("Desde".into()),
            TestMessage("TcpWriterActor".into()),
        ];
        let clone = messages.clone();

        for msg in clone {
            actor.do_send(msg);
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let mut buffer = vec![0u8; 512];
        let n = server_socket.read(&mut buffer).await.unwrap();
        let received = String::from_utf8_lossy(&buffer[..n]).to_string();

        let received_messages: Vec<_> = received
            .lines()
            .map(|line| TestMessage::from_representation(line.to_string()))
            .collect();

        assert_eq!(received_messages.len(), messages.len());
        for (i, msg) in received_messages.iter().enumerate() {
            assert_eq!(*msg, messages[i]);
        }
    }
}
