use crate::{
    IcuError,
    ic::api::{canister_self, msg_caller},
    memory,
};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error as ThisError;

///
/// AuthError
///

#[derive(CandidType, Debug, Deserialize, Serialize, ThisError)]
pub enum AuthError {
    #[error("{0}")]
    Custom(String),

    #[error("invalid error state - this should never happen")]
    InvalidState,

    #[error("the root canister is not defined")]
    NoRootDefined,

    #[error("one or more rules must be defined")]
    NoRulesDefined,

    #[error("there has to be a user canister defined in the schema")]
    NoUserCanister,

    #[error("this action is not allowed due to configuration settings")]
    NotAllowed,

    #[error("principal '{0}' does not match canister type '{1}'")]
    NotCanisterType(Principal, String),

    #[error("principal '{0}' is not a child of this canister")]
    NotChild(Principal),

    #[error("principal '{0}' is not a controller of this canister")]
    NotController(Principal),

    #[error("principal '{0}' is not the parent of this canister")]
    NotParent(Principal),

    #[error("principal '{0}' is not root")]
    NotRoot(Principal),

    #[error("principal '{0}' is not the current canister")]
    NotThis(Principal),
}

impl AuthError {
    #[must_use]
    pub fn custom(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

///
/// Rule
///

pub type RuleFn = Box<
    dyn Fn(Principal) -> Pin<Box<dyn Future<Output = Result<(), IcuError>> + Send>> + Send + Sync,
>;

pub type RuleResult = Pin<Box<dyn Future<Output = Result<(), IcuError>> + Send>>;

///
/// Auth Functions
///

// require_all
pub async fn require_all(rules: Vec<RuleFn>) -> Result<(), IcuError> {
    let caller = msg_caller();

    if rules.is_empty() {
        return Err(AuthError::NoRulesDefined.into());
    }

    for rule in rules {
        rule(caller).await?; // early return on failure
    }

    Ok(())
}

// require_any
pub async fn require_any(rules: Vec<RuleFn>) -> Result<(), IcuError> {
    let caller = msg_caller();

    if rules.is_empty() {
        return Err(AuthError::NoRulesDefined.into());
    }

    let mut last_error = None;
    for rule in rules {
        match rule(caller).await {
            Ok(()) => return Ok(()),
            Err(e) => last_error = Some(e),
        }
    }

    Err(last_error.unwrap_or_else(|| AuthError::InvalidState.into()))
}

///
/// RULE MACROS
///

#[macro_export]
macro_rules! auth_require_all {
    ($($f:path),* $(,)?) => {{
        $crate::auth::require_all(vec![
            $( Box::new(|pid| Box::pin($f(pid))) ),*
        ]).await
    }};
}

#[macro_export]
macro_rules! auth_require_any {
    ($($f:path),* $(,)?) => {{
        $crate::auth::require_any(vec![
            $( Box::new(|pid| Box::pin($f(pid))) ),*
        ]).await
    }};
}

///
/// RULE FUNCTIONS
///

// is_canister_type
// check caller against the id of a specific canister path
#[must_use]
pub fn is_canister_type(pid: Principal, canister: String) -> RuleResult {
    Box::pin(async move {
        memory::SubnetIndex::try_get_canister(&canister)
            .map_err(|_| AuthError::NotCanisterType(pid, canister.clone()))?;

        Ok(())
    })
}

// is_child
#[must_use]
pub fn is_child(pid: Principal) -> RuleResult {
    Box::pin(async move {
        memory::ChildIndex::get(&pid).ok_or(AuthError::NotChild(pid))?;

        Ok(())
    })
}

// is_controller
#[must_use]
pub fn is_controller(pid: Principal) -> RuleResult {
    Box::pin(async move {
        if crate::ic::api::is_controller(&pid) {
            Ok(())
        } else {
            Err(AuthError::NotController(pid).into())
        }
    })
}

// is_root
#[must_use]
pub fn is_root(pid: Principal) -> RuleResult {
    Box::pin(async move {
        let root_pid = memory::CanisterState::get_root_pid();

        if pid == root_pid {
            Ok(())
        } else {
            Err(AuthError::NotRoot(pid))?
        }
    })
}

// is_parent
#[must_use]
pub fn is_parent(pid: Principal) -> RuleResult {
    Box::pin(async move {
        if memory::CanisterState::has_parent_pid(&pid) {
            Ok(())
        } else {
            Err(AuthError::NotParent(pid))?
        }
    })
}

// is_same_canister
#[must_use]
pub fn is_same_canister(pid: Principal) -> RuleResult {
    Box::pin(async move {
        if pid == canister_self() {
            Ok(())
        } else {
            Err(AuthError::NotThis(pid))?
        }
    })
}
