pub trait Representable:
    Send + serde::Serialize + serde::de::DeserializeOwned + 'static + Sync
{
    fn as_representation(&self) -> String {
        serde_json::to_string(self).expect("Failed to get representation")
    }
    fn from_representation(repr: String) -> Self {
        serde_json::from_str(&repr).expect("Failed to build from representation")
    }
}
