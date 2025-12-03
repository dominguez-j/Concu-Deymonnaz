use crate::ipc::representable::Representable;
use actix::dev::ToEnvelope;
use actix::prelude::*;
use std::marker::PhantomData;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{LinesCodec, LinesCodecError};

pub struct TcpReaderActor<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    addressee: Addr<Addressee>,
    reader: Option<OwnedReadHalf>,
    _phantom: PhantomData<T>,
}

impl<Addressee, T> TcpReaderActor<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    pub fn new(reader: OwnedReadHalf, addressee: Addr<Addressee>) -> Self {
        Self {
            addressee: addressee,
            reader: Some(reader),
            _phantom: PhantomData,
        }
    }
}

impl<Addressee, T> Actor for TcpReaderActor<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let reader = self.reader.take().expect("reader missing");
        let framed = tokio_util::codec::FramedRead::new(reader, LinesCodec::new());
        ctx.add_stream(framed);
    }
}

impl<Addressee, T> StreamHandler<Result<String, LinesCodecError>> for TcpReaderActor<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    fn handle(&mut self, msg: Result<String, LinesCodecError>, _ctx: &mut Self::Context) {
        if let Ok(msg) = msg {
            self.addressee.do_send(T::from_representation(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::{
        io::AsyncWriteExt,
        net::{TcpListener, TcpStream},
    };

    #[derive(Debug, Clone, PartialEq, Message, Serialize, Deserialize)]
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
        let listener = TcpListener::bind("127.0.0.1:7000").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (read_half, _) = client.into_split();

        let received = Arc::new(tokio::sync::Mutex::new(Vec::<TestMessage>::new()));
        let addressee = TestAddressee {
            received: received.clone(),
        }
        .start();

        let _reader_actor = TcpReaderActor::new(read_half, addressee).start();

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

    #[actix::test]
    async fn test_tcp_reader_actor_handles_multiple_messages() {
        use std::sync::Mutex;

        struct ReceiverActor {
            received: Arc<Mutex<Vec<TestMessage>>>,
        }
        impl Actor for ReceiverActor {
            type Context = Context<Self>;
        }
        impl Handler<TestMessage> for ReceiverActor {
            type Result = ();
            fn handle(&mut self, msg: TestMessage, _ctx: &mut Self::Context) {
                self.received.lock().unwrap().push(msg);
            }
        }

        let listener = TcpListener::bind("127.0.0.1:7001").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let (mut server_socket, _) = listener.accept().await.unwrap();
        let (read_half, _) = client.into_split();

        let received = Arc::new(Mutex::new(Vec::<TestMessage>::new()));
        let receiver = ReceiverActor {
            received: received.clone(),
        }
        .start();

        let _reader_actor = TcpReaderActor::new(read_half, receiver).start();

        let messages = vec!["Hola", "Mundo", "Desde", "TcpReaderActor"];
        for msg in &messages {
            let json = serde_json::to_string(&TestMessage(msg.to_string())).unwrap();
            server_socket
                .write_all(format!("{}\n", json).as_bytes())
                .await
                .unwrap();
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let guard = received.lock().unwrap();
        assert_eq!(guard.len(), messages.len());

        for (i, msg) in guard.iter().enumerate() {
            assert_eq!(*msg, TestMessage(messages[i].to_string()));
        }
    }
}
