use candid::CandidType;
use ciborium::{Value, de::from_reader, ser::into_writer};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use thiserror::Error as ThisError;

///
/// Serialize/Deserialize
/// forces use of cbor (ciborium)
///

///
/// SerializeError
///

#[derive(CandidType, Debug, Serialize, Deserialize, ThisError)]
pub enum SerializeError {
    #[error("serialize error: {0}")]
    Serialize(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),
}

// serialize
pub fn serialize<T>(ty: &T) -> Result<Vec<u8>, SerializeError>
where
    T: Serialize,
{
    let mut writer = Vec::<u8>::new();
    into_writer(ty, &mut writer).map_err(|e| SerializeError::Serialize(e.to_string()))?;

    Ok(writer)
}

// deserialize
pub fn deserialize<T>(bytes: &[u8]) -> Result<T, SerializeError>
where
    T: DeserializeOwned,
{
    from_reader(bytes).map_err(|e| {
        // attempt to deserialize into a more generic Value for debugging
        match from_reader::<Value, _>(bytes) {
            Ok(value) => {
                SerializeError::Deserialize(format!("failed to deserialize: {e} ({value:?})"))
            }
            Err(debug_error) => SerializeError::Deserialize(format!(
                "failed to deserialize: {e}. DEBUG FAILED {debug_error}"
            )),
        }
    })
}
