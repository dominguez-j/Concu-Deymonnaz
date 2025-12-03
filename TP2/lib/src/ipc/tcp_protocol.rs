use actix::dev::ToEnvelope;
use actix::prelude::*;
use tokio::net::TcpStream;

use crate::ipc::representable::Representable;
use crate::ipc::tcp_reader::TcpReader;
use crate::ipc::tcp_writer::TcpWriter;

pub struct TcpProtocol<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    writer: Option<TcpWriter>,
    reader: Option<TcpReader<Addressee, T>>,
}

impl<Addressee, T> Default for TcpProtocol<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    fn default() -> Self {
        Self {
            writer: None,
            reader: None,
        }
    }
}

impl<Addressee, T> TcpProtocol<Addressee, T>
where
    Addressee: Actor + Handler<T>,
    <Addressee as Actor>::Context: ToEnvelope<Addressee, T>,
    T: Representable + Message<Result = ()> + Unpin,
{
    pub fn new(address: Addr<Addressee>, stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let writer = TcpWriter::new(writer);
        let mut reader = TcpReader::new(reader);
        reader.update_addressee(address);
        Self {
            writer: Some(writer),
            reader: Some(reader),
        }
    }
    pub fn build_stream(&mut self, stream: TcpStream) {
        let (reader, writer) = stream.into_split();
        self.writer = Some(TcpWriter::new(writer));
        self.reader = Some(TcpReader::new(reader));
    }
    pub fn build_addressee(&mut self, addressee: Addr<Addressee>) {
        if let Some(reader) = self.reader.as_mut() {
            reader.update_addressee(addressee);
        }
    }
    pub fn send<U>(&mut self, msg: U)
    where
        U: Representable + Message<Result = ()>,
    {
        if let Some(writer) = self.writer.as_mut() {
            writer.do_send(msg);
        }
    }
}
