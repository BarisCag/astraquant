use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    EncodeFailed(String),
    DecodeFailed(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodeFailed(msg) => write!(f, "Encode error: {}", msg),
            Self::DecodeFailed(msg) => write!(f, "Decode error: {}", msg),
        }
    }
}

pub(crate) fn bincode_config() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}

pub fn serialize_canonical<T: Serialize>(data: &T) -> Result<Vec<u8>, SerializationError> {
    bincode::serde::encode_to_vec(data, bincode_config())
        .map_err(|e| SerializationError::EncodeFailed(e.to_string()))
}

pub fn deserialize_canonical<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SerializationError> {
    let (val, _) = bincode::serde::decode_from_slice(bytes, bincode_config())
        .map_err(|e| SerializationError::DecodeFailed(e.to_string()))?;
    Ok(val)
}

pub fn serialize_event(event: &crate::events::AstraEvent) -> Result<Vec<u8>, SerializationError> {
    serialize_canonical(event)
}

pub fn deserialize_event(bytes: &[u8]) -> Result<crate::events::AstraEvent, SerializationError> {
    deserialize_canonical(bytes)
}
