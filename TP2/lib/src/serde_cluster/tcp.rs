use crate::messages::protocol::ProtocolMessage;
use crate::serde_cluster::json::{deserialize_json_str, serialize_json};

/// Serializa un mensaje TCP a String terminado en '\n'
pub fn serialize_tcp_msg(msg: &ProtocolMessage) -> String {
    serialize_json(msg) + "\n"
}

/// Deserializa una línea TCP (string) a ProtocolMessage
/// Haciendo trim \n al final
pub fn deserialize_tcp_msg(line: &str) -> ProtocolMessage {
    deserialize_json_str(line.trim())
}
