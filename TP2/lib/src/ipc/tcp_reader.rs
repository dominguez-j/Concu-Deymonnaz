use crate::ipc::representable::Representable;
use crate::ipc::tcp_reader_actor::TcpReaderActor;
use actix::dev::ToEnvelope;
use actix::prelude::*;
use std::ops::{Deref, DerefMut};
use tokio::net::tcp::OwnedReadHalf;

pub struct TcpReader<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    read_stream: Option<OwnedReadHalf>,
    actor: Option<Addr<TcpReaderActor<Addressee, T>>>,
}

impl<Addressee, T> TcpReader<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self {
            read_stream: Some(reader),
            actor: None,
        }
    }
    pub fn new_with_addressee(reader: OwnedReadHalf, addressee: Addr<Addressee>) -> Self {
        let actor = TcpReaderActor::new(reader, addressee).start();
        Self {
            read_stream: None,
            actor: Some(actor),
        }
    }
    pub fn update_addressee(&mut self, addressee: Addr<Addressee>) {
        self.actor = Some(TcpReaderActor::new(self.read_stream.take().unwrap(), addressee).start());
        self.read_stream = None;
    }
}

impl<Addressee, T> Deref for TcpReader<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    type Target = Option<Addr<TcpReaderActor<Addressee, T>>>;
    fn deref(&self) -> &Self::Target {
        &self.actor
    }
}

impl<Addressee, T> DerefMut for TcpReader<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::representable::Representable;
    use std::sync::Arc;
    use tokio::{io::AsyncWriteExt, net::TcpListener};

    #[derive(Debug, PartialEq, Message, serde::Serialize, serde::Deserialize)]
    #[rtype(result = "()")]
    struct TestMessage(pub String);
    impl Representable for TestMessage {}

    struct TestAddressee {
        pub received: Arc<tokio::sync::Mutex<Vec<TestMessage>>>,
    }
    impl Actor for TestAddressee {
        type Context = Context<Self>;
    }
    impl Handler<TestMessage> for TestAddressee {
        type Result = ();
        fn handle(&mut self, msg: TestMessage, _ctx: &mut Self::Context) {
            let data = self.received.clone();
            actix::spawn(async move {
                data.lock().await.push(msg);
            });
        }
    }

    #[actix::test]
    async fn test_tcp_reader_actor_receives_and_deserializes_correctly() {
        let listener = TcpListener::bind("127.0.0.1:7002").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (read_half, _) = client.into_split();

        let received = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<TestMessage>::new()));
        let addressee = TestAddressee {
            received: received.clone(),
        }
        .start();

        let _reader_actor = TcpReader::new_with_addressee(read_half, addressee);

        let msg = TestMessage("Hola".to_string()).as_representation();
        server_socket
            .write_all(format!("{}\n", msg).as_bytes())
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let guard = received.lock().await;
        assert_eq!(guard.len(), 1);
        assert_eq!(
            guard[0],
            TestMessage::from_representation("\"Hola\"".to_string())
        );
    }
}
