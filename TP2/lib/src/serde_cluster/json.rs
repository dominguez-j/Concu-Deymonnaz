use crate::messages::protocol::ProtocolMessage;

/// Serializa un mensaje a JSON (String)
pub fn serialize_json(msg: &ProtocolMessage) -> String {
    serde_json::to_string(msg).expect("serde json to string")
}

/// Serializa un mensaje a bytes JSON (Vec<u8>)
pub fn serialize_json_bytes(msg: &ProtocolMessage) -> Vec<u8> {
    serde_json::to_vec(msg).expect("serde json to vec")
}

/// Deserializa un mensaje desde un &str
pub fn deserialize_json_str(data: &str) -> ProtocolMessage {
    serde_json::from_str(data).expect("serde json from string")
}

/// Deserializa un mensaje desde un slice de bytes
pub fn deserialize_json_bytes(data: &[u8]) -> ProtocolMessage {
    let recv_msg = String::from_utf8(data.to_vec()).expect("serde json to string");
    serde_json::from_str(&recv_msg).expect("serde json from string")
}
