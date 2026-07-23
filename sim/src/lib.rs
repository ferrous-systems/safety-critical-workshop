#![no_std]

use core::time::Duration;

use cortex_m_rt::exception;
use mantra_macros::req_link;
use nrf_hal::Board;

/// Our custom panic handler.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    nrf_hal::fail();
}

/// The default HardFault handler just spins, so replace it.
#[exception]
unsafe fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    defmt::error!("HardFault!");
    nrf_hal::fail();
}

// this prevents the panic message being printed *twice* when `defmt::panic!` is invoked
#[defmt::panic_handler]
fn defmt_panic() -> ! {
    nrf_hal::fail();
}

defmt::timestamp!("{=u64:tus}", nrf_hal::uptime_us());

pub const UPDATE_DELAY_MS: u64 = 50;

const MAX_WAIT_TIME_SEC: u64 = 5;

pub struct Sim {
    board: Board,
    expected_mode: RadMode,
}

impl Sim {
    pub fn init() -> Self {
        let mut sim = Self {
            board: Board::init().expect("Failed to initialize the board"),
            expected_mode: RadMode::Idle,
        };

        sim.set_door_sensor(OutputState::Off);
        sim.set_environment_confirmation(OutputState::Off);
        sim.set_radiation_state(RadiationState::Deactive);
        sim.set_start_stop(StartStopState::Stop);

        sim.wait_update();

        sim.board.print_io();

        sim
    }

    /// Bring *RAD* I/O production mode.
    ///
    /// **Note:** This internally calls `Self::update` to set the *RAD* I/O for production mode.
    #[req_link("rad.sw.operation.start", "rad.sw.operation.pre-condition")]
    pub fn rad_to_production(&mut self) {
        self.update(); // ensure we read the latest input state

        assert_eq!(
            self.expected_mode,
            RadMode::Idle,
            "RAD may only be changed to 'operation' if it is expected to be in 'idle'"
        );
        assert_eq!(self.actual_mode(), RadMode::Idle, "RAD was not in 'idle'");

        self.set_door_sensor(OutputState::On);
        self.set_environment_confirmation(OutputState::On);
        self.set_start_stop(StartStopState::Start);

        let sys_time = self.sys_time();
        self.expected_mode = RadMode::Operation;

        loop {
            if self.start_request_detected() && self.actual_mode() == self.expected_mode {
                break;
            } else if sys_time.abs_diff(self.sys_time()).as_secs() > MAX_WAIT_TIME_SEC {
                panic!("RAD did not detect start request after ~{MAX_WAIT_TIME_SEC}s");
            }

            self.update();
        }

        let sys_time = self.sys_time();
        loop {
            if self.radiation_relay() == OutputState::On {
                break;
            } else if sys_time.abs_diff(self.sys_time()).as_secs() > MAX_WAIT_TIME_SEC {
                panic!("RAD did not activate radiation relay after ~{MAX_WAIT_TIME_SEC}s");
            }

            self.update();
        }

        assert_eq!(
            self.radiation_relay(),
            OutputState::On,
            "Radiation relay should be active"
        );

        self.set_radiation_state(RadiationState::Active);

        self.wait_update();

        // Ensure we are still in production
        assert_eq!(
            self.actual_mode(),
            self.expected_mode,
            "RAD should be in operation mode"
        );
    }

    pub fn rad_to_idle(&mut self) {
        self.update(); // ensure we read the latest input state

        assert_eq!(
            self.expected_mode,
            RadMode::Operation,
            "RAD may only be changed to 'idle' if it is expected to be in 'operation'"
        );
        assert_eq!(
            self.actual_mode(),
            RadMode::Operation,
            "RAD was not in 'operation'"
        );

        // Requesting a stop
        self.set_start_stop(StartStopState::Stop);

        self.wait_update();

        let sys_time = self.sys_time();
        loop {
            // RAD should have turned of radiation with requested stop
            if self.radiation_relay() == OutputState::Off {
                self.set_radiation_state(RadiationState::Deactive);
            }

            if self.actual_mode() == RadMode::Idle {
                break;
            } else if sys_time.abs_diff(self.sys_time()).as_secs() > MAX_WAIT_TIME_SEC {
                panic!("RAD did go back to 'idle' after ~{MAX_WAIT_TIME_SEC}s");
            }

            self.update();
        }

        self.expected_mode = RadMode::Idle;
        assert_eq!(
            self.actual_mode(),
            RadMode::Idle,
            "RAD did not go back to 'idle'"
        );
    }

    #[req_link("rad.hw.start-stop-switch")]
    pub fn set_start_stop(&mut self, state: StartStopState) {
        match state {
            StartStopState::Start => {
                self.board.dig_out.p1_01.set_high();
                self.board.leds._1.on();
            }
            StartStopState::Stop => {
                self.board.dig_out.p1_01.set_low();
                self.board.leds._1.off();
            }
        }
    }

    pub fn start_stop_state(&self) -> StartStopState {
        if self.board.dig_out.p1_01.is_set_high() {
            StartStopState::Start
        } else {
            StartStopState::Stop
        }
    }

    #[req_link("rad.hw.door-sensor")]
    pub fn set_door_sensor(&mut self, state: OutputState) {
        match state {
            OutputState::On => {
                self.board.dig_out.p1_02.set_high();
                self.board.leds._2.on();
            }
            OutputState::Off => {
                self.board.dig_out.p1_02.set_low();
                self.board.leds._2.off();
            }
        }
    }

    pub fn door_sensor(&self) -> OutputState {
        if self.board.dig_out.p1_02.is_set_high() {
            OutputState::On
        } else {
            OutputState::Off
        }
    }

    #[req_link("rad.hw.confirmation-switch")]
    pub fn set_environment_confirmation(&mut self, state: OutputState) {
        match state {
            OutputState::On => {
                self.board.dig_out.p1_03.set_high();
                self.board.leds._3.on();
            }
            OutputState::Off => {
                self.board.dig_out.p1_03.set_low();
                self.board.leds._3.off();
            }
        }
    }

    pub fn confirmation_state(&self) -> OutputState {
        if self.board.dig_out.p1_03.is_set_high() {
            OutputState::On
        } else {
            OutputState::Off
        }
    }

    #[req_link("rad.hw.radiation-sensor")]
    pub fn set_radiation_state(&mut self, state: RadiationState) {
        match state {
            RadiationState::Active => {
                self.board.dig_out.p1_04.set_high();
                self.board.leds._4.on();
            }
            RadiationState::Deactive => {
                self.board.dig_out.p1_04.set_low();
                self.board.leds._4.off();
            }
        }
    }

    pub fn radiation_state(&self) -> RadiationState {
        if self.board.dig_out.p1_04.is_set_high() {
            RadiationState::Active
        } else {
            RadiationState::Deactive
        }
    }

    pub fn radiation(&self) -> i16 {
        self.board.analog_in.read()
    }

    #[req_link("rad.hw.radiation-relay")]
    pub fn radiation_relay(&self) -> OutputState {
        if self.board.dig_in.p1_05.is_high() {
            OutputState::On
        } else {
            OutputState::Off
        }
    }

    #[req_link("rad.hw.mode-indicator")]
    pub fn actual_mode(&self) -> RadMode {
        if self.board.dig_in.p1_06.is_high() {
            RadMode::Operation
        } else {
            RadMode::Idle
        }
    }

    pub fn start_request_detected(&self) -> bool {
        self.board.dig_in.p1_07.is_high()
    }

    pub fn update(&mut self) {
        self.board.update();
    }

    /// Wait for the demo to update its I/O
    pub fn wait_update(&mut self) {
        self.update();

        self.wait(Duration::from_millis(UPDATE_DELAY_MS)); // make sure RAD device received the update

        self.update();
    }

    pub fn wait(&mut self, duration: Duration) {
        self.board.timer.wait(duration);
    }

    pub fn sys_time(&self) -> Duration {
        self.board.sys_time()
    }

    pub fn print(&self) {
        defmt::info!("Start/Stop switch (P1.01) in '{}'", self.start_stop_state());
        defmt::info!("Door Sensor (P1.02) is '{}'", self.door_sensor());
        defmt::info!("Confirmation (P1.03) is '{}'", self.confirmation_state());
        defmt::info!("Radiation (P0.03) is '{}'", self.radiation());
        defmt::info!("Radiation relay (P1.05) is '{}'", self.radiation_relay());
        defmt::info!("Rad (P1.06) in '{}' mode", self.actual_mode());
        defmt::info!(
            "Start requested detected (P1.07) is '{}'",
            self.start_request_detected()
        );
    }

    pub fn board(&mut self) -> &mut Board {
        &mut self.board
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum OutputState {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum StartStopState {
    Start,
    Stop,
}

#[req_link("rad.mode")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RadMode {
    Idle,
    Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RadiationState {
    Active,
    Deactive,
}
