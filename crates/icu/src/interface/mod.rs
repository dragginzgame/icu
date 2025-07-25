pub mod cascade;
pub mod ic;
pub mod request;
pub mod response;
pub mod state;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

///
/// InterfaceError
///

#[derive(CandidType, Debug, Deserialize, Serialize, ThisError)]
pub enum InterfaceError {
    #[error(transparent)]
    IcError(#[from] ic::IcError),

    #[error(transparent)]
    RequestError(#[from] request::RequestError),
}
