#![cfg_attr(not(test), no_std)]

use mantra_macros::req;

#[cfg(feature = "hw")]
use defmt::assert;

#[cfg_attr(test, mockall_double::double)]
use crate::hal::Hal;

mod hal;

#[cfg(test)]
mod tests;

/// The *RAD* control structure.
pub struct Rad {
    /// Current mode *RAD* is in
    mode: Mode,
    /// HAL for *RAD*
    hal: Hal,
}

impl Rad {
    /// Initialize *RAD*.
    ///
    /// Starts in `idle` mode.
    pub fn init(hal: Hal) -> Self {
        Self {
            mode: Mode::Idle,
            hal,
        }
    }

    pub fn run(&mut self) -> ! {
        let mut start_triggered = false;
        let mut stop_triggered = false;
        let mut prev_mode = None;

        loop {
            match self.mode {
                Mode::Idle => mantra_macros::impl_req!("rad.sw.idle" => {
                    assert!(!self.hal.radiation_active(), "Radiation must not be active in `idle` mode");

                    mantra_macros::impl_req!("rad.sw.operation.start" => {
                        if self.hal.start_requested() && !start_triggered {
                            start_triggered = true;
                        } else if !self.hal.start_requested() && start_triggered {
                            start_triggered = false;
                        }

                        if start_triggered {
                            match self.operation_conditions_fulfilled() {
                                Ok(_) => {
                                    start_triggered = false;
                                    prev_mode = Some(self.mode);

                                    self.mode = Mode::Operation
                                },
                                Err(err) => {
                                    #[cfg(feature = "hw")]
                                    defmt::warn!("Could not start operation due to error: '{}'", err);
                                }
                            }
                        }
                    })
                }),
                Mode::Operation => mantra_macros::impl_req!("rad.sw.operation" => {
                    mantra_macros::impl_req!("rad.sw.operation.stop" => {
                        if self.hal.stop_requested() && !stop_triggered {
                            stop_triggered = true;
                        }

                        if stop_triggered {
                            self.hal.stop_radiation();
                        }

                        mantra_macros::impl_req!("rad.sw.operation.post-condition" => {
                            if !self.hal.radiation_active() {
                                self.mode = Mode::Idle;
                                prev_mode = Some(Mode::Operation);
                                stop_triggered = false;
                            }
                        })
                    });

                    // we entered operation mode => start RAD
                    if !stop_triggered && prev_mode == Some(Mode::Idle) {
                        self.hal.start_radiation();
                    }
                }),
            }
        }
    }

    #[req("rad.sw.operation.pre-condition")]
    fn operation_conditions_fulfilled(&self) -> Result<(), RadError> {
        if !self.hal.entrance_door_closed() {
            return Err(RadError::EntranceDoorOpen);
        }

        if !self.hal.safe_environment_confirmed() {
            return Err(RadError::MissingSafeEnvironmentConfirmation);
        }

        Ok(())
    }
}

/// Possible modes of the *RAD* product.
#[req("rad.mode")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// *RAD* is waiting for input. No radiation is output
    Idle,
    /// *RAD* is performing radiation therapy
    Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RadError {
    EntranceDoorOpen,
    MissingSafeEnvironmentConfirmation,
}
