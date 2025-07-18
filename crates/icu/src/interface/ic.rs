use crate::{
    Error, Log,
    helper::{format_cycles, get_wasm_hash},
    ic::{
        call::{CallFailed, CandidDecodeFailed, Error as CallError},
        mgmt::{
            self, CanisterInstallMode, CanisterSettings, CanisterStatusArgs, CanisterStatusResult,
            CreateCanisterArgs, DepositCyclesArgs, InstallCodeArgs, WasmModule,
        },
    },
    interface::InterfaceError,
    log,
};
use candid::{CandidType, Error as CandidError, Principal};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

///
/// IcError
///

#[derive(CandidType, Debug, Deserialize, Serialize, ThisError)]
pub enum IcError {
    #[error("call rejected: {0}")]
    CallFailed(String),

    #[error("candid error: {0}")]
    CandidError(String),

    #[error("candid error: {0}")]
    CandidDecodeFailed(String),

    #[error("wasm hash matches")]
    WasmHashMatches,
}

impl From<CallFailed> for IcError {
    fn from(error: CallFailed) -> Self {
        Self::CallFailed(error.to_string())
    }
}

impl From<CandidError> for IcError {
    fn from(error: CandidError) -> Self {
        Self::CandidError(error.to_string())
    }
}

impl From<CandidDecodeFailed> for IcError {
    fn from(error: CandidDecodeFailed) -> Self {
        Self::CandidDecodeFailed(error.to_string())
    }
}

impl From<CallError> for IcError {
    fn from(error: CallError) -> Self {
        Self::CallFailed(error.to_string())
    }
}

// canister_self
#[must_use]
pub fn canister_self() -> Principal {
    crate::ic::api::canister_self()
}

// canister_status
pub async fn canister_status(canister_pid: Principal) -> Result<CanisterStatusResult, Error> {
    let args = CanisterStatusArgs {
        canister_id: canister_pid,
    };
    let res = mgmt::canister_status(&args)
        .await
        .map_err(IcError::from)
        .map_err(InterfaceError::IcError)?;

    Ok(res)
}

// deposit_cycles
pub async fn deposit_cycles(canister_pid: Principal, cycles: u128) -> Result<(), Error> {
    let args = DepositCyclesArgs {
        canister_id: canister_pid,
    };
    mgmt::deposit_cycles(&args, cycles)
        .await
        .map_err(IcError::from)
        .map_err(InterfaceError::IcError)?;

    Ok(())
}

// install_code
pub async fn install_code(args: &InstallCodeArgs) -> Result<(), Error> {
    mgmt::install_code(args)
        .await
        .map_err(IcError::from)
        .map_err(InterfaceError::IcError)?;

    Ok(())
}

// module_hash
pub async fn module_hash(canister_id: Principal) -> Result<Option<Vec<u8>>, Error> {
    let response = canister_status(canister_id).await?;

    Ok(response.module_hash)
}

///
/// ic_create_canister
///

pub async fn ic_create_canister(
    name: &str,
    bytes: &[u8],
    controllers: Vec<Principal>,
    arg: Vec<u8>,
) -> Result<Principal, Error> {
    //
    // create canister
    //

    let cycles = 5_000_000_000_000;
    let settings = Some(CanisterSettings {
        controllers: Some(controllers),
        ..Default::default()
    });
    let cc_args = CreateCanisterArgs { settings };
    let canister_pid = mgmt::create_canister_with_extra_cycles(&cc_args, cycles)
        .await
        .map_err(IcError::from)
        .map_err(InterfaceError::IcError)?
        .canister_id;

    //
    // install code
    //

    let install_args = InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id: canister_pid,
        wasm_module: WasmModule::from(bytes),
        arg,
    };
    mgmt::install_code(&install_args)
        .await
        .map_err(IcError::from)
        .map_err(InterfaceError::IcError)?;

    //
    // debug
    //

    #[allow(clippy::cast_precision_loss)]
    let bytes_fmt = bytes.len() as f64 / 1_000.0;
    log!(
        Log::Ok,
        "ic.create_canister: {} created ({} KB) {} with {}",
        name,
        bytes_fmt,
        canister_pid,
        format_cycles(cycles)
    );

    Ok(canister_pid)
}

/// ic_upgrade_canister
pub async fn ic_upgrade_canister(canister_pid: Principal, bytes: &[u8]) -> Result<(), Error> {
    // module_hash
    let module_hash = module_hash(canister_pid).await?;
    if module_hash == Some(get_wasm_hash(bytes)) {
        Err(InterfaceError::IcError(IcError::WasmHashMatches))?;
    }

    // args
    let install_args = InstallCodeArgs {
        mode: CanisterInstallMode::Upgrade(None),
        canister_id: canister_pid,
        wasm_module: WasmModule::from(bytes),
        arg: vec![],
    };
    mgmt::install_code(&install_args)
        .await
        .map_err(|e| IcError::CallFailed(e.to_string()))
        .map_err(InterfaceError::IcError)?;

    // debug
    #[allow(clippy::cast_precision_loss)]
    let bytes_fmt = bytes.len() as f64 / 1_000.0;
    log!(
        Log::Ok,
        "canister_upgrade: {} ({} KB) upgraded",
        canister_pid,
        bytes_fmt,
    );

    Ok(())
}
