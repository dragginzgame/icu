use crate::ic::structures::{BTreeMap, DefaultMemory};
use candid::{CandidType, Principal};
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

///
/// SubnetIndexError
///

#[derive(CandidType, Debug, Deserialize, Serialize, ThisError)]
pub enum SubnetIndexError {
    #[error("canister not found: {0}")]
    CanisterNotFound(String),
}

//
// SUBNET_INDEX
//

///
/// SubnetIndex
///

#[derive(Deref, DerefMut)]
pub struct SubnetIndex(BTreeMap<String, Principal>);

impl SubnetIndex {
    #[must_use]
    pub fn init(memory: DefaultMemory) -> Self {
        Self(BTreeMap::init(memory))
    }

    #[must_use]
    pub fn get_data(&self) -> Vec<(String, Principal)> {
        self.iter().collect()
    }

    pub fn set_data(&mut self, data: Vec<(String, Principal)>) {
        self.clear();
        for (k, v) in data {
            self.insert(k, v);
        }
    }

    #[must_use]
    pub fn get_canister(&self, path: &str) -> Option<Principal> {
        self.get(&path.to_string())
    }

    pub fn try_get_canister(&self, path: &str) -> Result<Principal, SubnetIndexError> {
        self.get_canister(path)
            .ok_or_else(|| SubnetIndexError::CanisterNotFound(path.to_string()))
    }

    pub fn set_canister(&mut self, path: &str, id: Principal) {
        self.insert(path.to_string(), id);
    }
}

///
/// SubnetIndexData
///

pub type SubnetIndexData = Vec<(String, Principal)>;
