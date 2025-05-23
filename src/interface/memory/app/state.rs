use crate::{Error, memory::MemoryError};

pub use crate::memory::{
    APP_STATE,
    app::{AppCommand, AppMode, AppState, AppStateData},
};

// command
pub fn command(cmd: AppCommand) -> Result<(), Error> {
    APP_STATE
        .with_borrow_mut(|this| this.command(cmd))
        .map_err(MemoryError::AppStateError)?;

    Ok(())
}

// get_data
#[must_use]
pub fn get_data() -> AppStateData {
    APP_STATE.with_borrow(AppState::get_data)
}

// set_data
pub fn set_data(data: AppStateData) -> Result<(), Error> {
    APP_STATE
        .with_borrow_mut(|this| this.set_data(data))
        .map_err(MemoryError::AppStateError)?;

    Ok(())
}

// get_mode
#[must_use]
pub fn get_mode() -> AppMode {
    APP_STATE.with_borrow(AppState::get_mode)
}
