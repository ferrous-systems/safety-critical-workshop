#![cfg_attr(not(test), no_std)]

use mantra_macros::req;

#[cfg(feature = "hw")]
use defmt::assert;

#[cfg_attr(test, mockall_double::double)]
use crate::hal::Hal;
use crate::hal::LedIndicator;

mod hal;

#[cfg(test)]
mod tests;

/// The *RAD* control structure.
pub struct Rad {
    /// Current mode *RAD* is in
    mode: RadMode,
    start_triggered: bool,
    stop_triggered: bool,
    prev_mode: Option<RadMode>,
}

impl Rad {
    /// Initialize *RAD*.
    ///
    /// Starts in `idle` mode.
    pub fn init() -> Self {
        Self {
            mode: RadMode::Idle,
            start_triggered: false,
            stop_triggered: false,
            prev_mode: None,
        }
    }

    pub fn update(&mut self, hal: &mut impl Hal) {
        match self.mode {
            RadMode::Idle => mantra_macros::impl_req!("rad.sw.idle" => {
                assert!(!hal.radiation_active(), "Radiation must not be active in `idle` mode");

                mantra_macros::impl_req!("rad.sw.operation.start" => {
                    if hal.start_requested() && !self.start_triggered {
                        self.start_triggered = true;
                    } else if !hal.start_requested() && self.start_triggered {
                        self.start_triggered = false;
                    }

                    if self.start_triggered {
                        match operation_conditions_fulfilled(hal) {
                            Ok(_) => {
                                self.start_triggered = false;
                                self.prev_mode = Some(self.mode);

                                self.mode = RadMode::Operation
                            },
                            Err(err) => {
                                #[cfg(feature = "hw")]
                                defmt::warn!("Could not start operation due to error: '{}'", err);
                            }
                        }
                    }
                })
            }),
            RadMode::Operation => mantra_macros::impl_req!("rad.sw.operation" => {
                mantra_macros::impl_req!("rad.sw.operation.stop" => {
                    if hal.stop_requested() && !self.stop_triggered {
                        self.stop_triggered = true;
                    }

                    if self.stop_triggered {
                        hal.stop_radiation();
                    }

                    mantra_macros::impl_req!("rad.sw.operation.post-condition" => {
                        if !hal.radiation_active() {
                            self.mode = RadMode::Idle;
                            self.prev_mode = Some(RadMode::Operation);
                            self.stop_triggered = false;
                        }
                    })
                });

                // we entered operation mode => start RAD
                if !self.stop_triggered && self.prev_mode == Some(RadMode::Idle) {
                    hal.start_radiation();
                }
            }),
        }

        self.update_indicators(hal);
    }

    #[req("rad.sw.indicator")]
    fn update_indicators(&self, hal: &mut impl Hal) {
        match self.mode {
            RadMode::Idle => {
                hal.set_mode_indicator(LedIndicator::Off);
            }
            RadMode::Operation => {
                hal.set_mode_indicator(LedIndicator::On);
            }
        }

        if hal.entrance_door_closed() {
            hal.set_entrance_door_indicator(LedIndicator::On);
        } else {
            hal.set_entrance_door_indicator(LedIndicator::Off);
        }

        if hal.safe_environment_confirmed() {
            hal.set_confirmation_switch_indicator(LedIndicator::On);
        } else {
            hal.set_confirmation_switch_indicator(LedIndicator::Off);
        }

        if hal.radiation_active() {
            hal.set_radiation_relay_indicator(LedIndicator::On);
        } else {
            hal.set_radiation_relay_indicator(LedIndicator::Off);
        }
    }
}

#[req("rad.sw.operation.pre-condition")]
fn operation_conditions_fulfilled(hal: &impl Hal) -> Result<(), RadError> {
    if !hal.entrance_door_closed() {
        return Err(RadError::EntranceDoorOpen);
    }

    if !hal.safe_environment_confirmed() {
        return Err(RadError::MissingSafeEnvironmentConfirmation);
    }

    Ok(())
}

/// Possible modes of the *RAD* product.
#[req("rad.mode")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadMode {
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
