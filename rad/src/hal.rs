use mantra_macros::{req, req_link};
#[cfg(test)]
use mockall::automock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedIndicator {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputState {
    On,
    Off,
}

#[req("rad.sw.hal")]
#[cfg_attr(test, automock)]
pub trait Hal {
    /// [req_link("rad.hw.start-stop-switch")]
    fn start_requested(&self) -> bool;

    /// [req_link("rad.hw.start-stop-switch")]
    fn stop_requested(&self) -> bool;

    /// [req_link("rad.hw.radiation-relay")]
    fn start_radiation(&mut self);

    /// [req_link("rad.hw.radiation-relay")]
    fn stop_radiation(&mut self);

    /// [req_link("rad.hw.radiation-relay")]
    fn radiation_output_state(&self) -> OutputState;

    /// [req_link("rad.hw.radiation-sensor")]
    fn radiation_active(&self) -> bool;

    /// [req_link("rad.hw.door-sensor")]
    fn entrance_door_closed(&self) -> bool;

    /// [req_link("rad.hw.confirmation-switch")]
    fn safe_environment_confirmed(&self) -> bool;

    /// [req_link("rad.hw.mode-indicator")]
    fn set_mode_indicator(&mut self, indicator: LedIndicator);

    /// [req_link("rad.hw.door-sensor.status-LED")]
    fn set_entrance_door_indicator(&mut self, indicator: LedIndicator);

    /// [req_link("rad.hw.confirmation-switch.status-LED")]
    fn set_confirmation_switch_indicator(&mut self, indicator: LedIndicator);

    /// [req_link("rad.hw.radiation-relay.status-LED")]
    fn set_radiation_output_indicator(&mut self, indicator: LedIndicator);

    fn set_start_stop_indicator(&mut self);
}

#[cfg(feature = "hw")]
macro_rules! led_ctrl {
    ($mode:ident, $led:expr) => {
        match $mode {
            LedIndicator::On => $led.on(),
            LedIndicator::Off => $led.off(),
        }
    };
}

#[cfg(feature = "hw")]
impl Hal for nrf_hal::Board {
    #[req_link("rad.hw.start-stop-switch")]
    fn start_requested(&self) -> bool {
        self.dig_in.p1_05.is_low()
    }

    #[req_link("rad.hw.start-stop-switch")]
    fn stop_requested(&self) -> bool {
        self.dig_in.p1_05.is_high()
    }

    #[req_link("rad.hw.radiation-relay")]
    fn start_radiation(&mut self) {
        defmt::info!("Starting radiation output");
        self.dig_out.p1_01.set_low();
    }

    #[req_link("rad.hw.radiation-relay")]
    fn stop_radiation(&mut self) {
        defmt::info!("Stopping radiation output");
        self.dig_out.p1_01.set_high();
    }

    #[req_link("rad.hw.radiation-relay")]
    fn radiation_output_state(&self) -> OutputState {
        if self.dig_out.p1_01.is_set_low() {
            OutputState::On
        } else {
            OutputState::Off
        }
    }

    #[req_link("rad.hw.radiation-sensor")]
    fn radiation_active(&self) -> bool {
        self.dig_in.p1_08.is_low()
    }

    #[req_link("rad.hw.door-sensor")]
    fn entrance_door_closed(&self) -> bool {
        self.dig_in.p1_06.is_low()
    }

    #[req_link("rad.hw.confirmation-switch")]
    fn safe_environment_confirmed(&self) -> bool {
        self.dig_in.p1_07.is_low()
    }

    #[req_link("rad.hw.mode-indicator")]
    fn set_mode_indicator(&mut self, indicator: LedIndicator) {
        led_ctrl!(indicator, self.leds._1);

        // Used to inform the simulation in which mode the RAD is in
        #[cfg(feature = "hw-testing")]
        match indicator {
            LedIndicator::On => self.dig_out.p1_02.set_low(),
            LedIndicator::Off => self.dig_out.p1_02.set_high(),
        }
    }

    #[req_link("rad.hw.door-sensor.status-LED")]
    fn set_entrance_door_indicator(&mut self, indicator: LedIndicator) {
        led_ctrl!(indicator, self.leds._2)
    }

    #[req_link("rad.hw.confirmation-switch.status-LED")]
    fn set_confirmation_switch_indicator(&mut self, indicator: LedIndicator) {
        led_ctrl!(indicator, self.leds._3)
    }

    #[req_link("rad.hw.radiation-relay.status-LED")]
    fn set_radiation_output_indicator(&mut self, indicator: LedIndicator) {
        led_ctrl!(indicator, self.leds._4)
    }

    fn set_start_stop_indicator(&mut self) {
        if self.start_requested() {
            self.dig_out.p1_03.set_low();
        } else {
            self.dig_out.p1_03.set_high();
        }
    }
}
