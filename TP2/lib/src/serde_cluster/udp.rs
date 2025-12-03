use crate::messages::protocol::ProtocolMessage;
use crate::serde_cluster::json::{deserialize_json_bytes, serialize_json_bytes};

/// Serializa un mensaje UDP a Vec<u8>
pub fn serialize_udp_msg(msg: &ProtocolMessage) -> Vec<u8> {
    serialize_json_bytes(msg)
}

/// Deserializa bytes UDP a ProtocolMessage
pub fn deserialize_udp_msg(buf: &[u8], size: usize) -> ProtocolMessage {
    let slice = &buf[..size];
    deserialize_json_bytes(slice)
}
