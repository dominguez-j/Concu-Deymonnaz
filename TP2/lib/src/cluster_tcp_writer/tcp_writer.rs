use crate::log;
use actix::{Actor, Context, Handler, Message, prelude::*};
use actix_async_handler::async_handler;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ShutdownTcpWriter;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WriteMessage {
    pub line: String,
}
pub struct TcpWriter {
    pub write: Option<WriteHalf<TcpStream>>,
}

impl Actor for TcpWriter {
    type Context = Context<Self>;
}

#[async_handler]
impl Handler<WriteMessage> for TcpWriter {
    type Result = ();
    async fn handle(&mut self, msg: WriteMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut write = self.write.take().expect("non atomic!?");

        let ret_write = async move {
            if let Ok(_) = write.write_all((msg.line).as_bytes()).await {
            } else {
                log!("[TCPWRITER] Write failed");
            }
            write
        }
        .await;

        self.write = Some(ret_write);
    }
}

impl Handler<ShutdownTcpWriter> for TcpWriter {
    type Result = ();
    fn handle(&mut self, _msg: ShutdownTcpWriter, ctx: &mut Self::Context) {
        ctx.stop();
    }
}
