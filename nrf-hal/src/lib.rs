//! Minimal Board Support Package (BSP) for the nRF52840 Development Kit
//! See <https://www.nordicsemi.com/Products/Development-hardware/nrf52840-dk>
//!
//! Based on `nrf52-code` from Ferrous Systems [rust-exercises](https://github.com/ferrous-systems/rust-exercises).

// #![deny(missing_docs)]
// #![deny(warnings)]
#![no_std]

use core::{
    hint::spin_loop,
    sync::atomic::{self, AtomicBool, AtomicU32, Ordering},
    time::Duration,
};

use cortex_m_semihosting::debug;
use embedded_hal::delay::DelayNs;
pub use hal;
pub use hal::pac::interrupt;
use hal::{
    gpio::{Input, Level, Output, OutputDrive, Port},
    interrupt::{typelevel, InterruptExt},
    pac::saadc::{regs::Samplerate, vals::SamplerateMode},
    Peri,
};

use defmt_rtt as _; // global logger

/// Components on the board
pub struct Board {
    /// LEDs
    pub leds: Leds,
    /// Buttons.
    pub buttons: Buttons,
    /// Timer
    pub timer: Timer,
    /// digital inputs
    pub dig_in: DigitalInputs,
    /// digital outputs
    pub dig_out: DigitalOutputs,
    pub analog_in: AnalogInput<'static>,
}

pub struct AnalogInput<'d> {
    _p: Peri<'d, hal::peripherals::SAADC>,
    debounce_delay_us: u64,
    last_change_time_us: u64,
    val: i16,
    buf: &'static mut SaadcBuffer,
}

// Code adapted from: https://github.com/embassy-rs/embassy/blob/main/embassy-nrf/src/saadc.rs
impl<'d> AnalogInput<'d> {
    pub fn new(saadc: Peri<'d, hal::peripherals::SAADC>) -> Self {
        let buf = cortex_m::singleton!(: SaadcBuffer = SaadcBuffer([0; SAADC_BUFFER_SIZE]))
            .expect("Only one AnalogInput supported");
        let r = hal::pac::SAADC;
        r.enable().write(|w| w.set_enable(true));

        let oversample = hal::pac::saadc::vals::Oversample::Over2x;
        // spec p. 678 f_sample < 1 / (t_acq + t_conv) and p. 713 f_sample = 16 MHz / <CC value> with CC in 80..2047
        // t_conv < 2µs see electrical spec table on p. 714
        // 16 MHz / 124 = ~120_000 < 1 / (3µs + 2µs) = ~200_000
        let mut samplerate = Samplerate::default();
        samplerate.set_mode(SamplerateMode::Timers);
        samplerate.set_cc(124);
        r.samplerate().write_value(samplerate);

        r.resolution()
            .write(|w| w.set_val(hal::pac::saadc::vals::Val::_12bit));
        r.oversample().write(|w| w.set_oversample(oversample));

        // Configure channel for p0.03
        r.ch(0).pselp().write(|w| {
            w.set_pselp(hal::pac::saadc::vals::Psel::AnalogInput0);
        });
        r.ch(0).config().write(|w| {
            w.set_refsel(hal::pac::saadc::vals::Refsel::Vdd14);
            w.set_gain(hal::pac::saadc::vals::Gain::Gain1);
            w.set_tacq(hal::pac::saadc::vals::Tacq::_3us);
            w.set_mode(hal::pac::saadc::vals::ConfigMode::Se);
            w.set_resp(hal::pac::saadc::vals::Resp::Pullup);
            w.set_resn(hal::pac::saadc::vals::Resn::Bypass);
            w.set_burst(!matches!(
                oversample,
                hal::pac::saadc::vals::Oversample::Bypass
            ));
        });

        // Disable all events interrupts
        r.intenclr().write(|w| w.0 = 0x003F_FFFF);

        interrupt::SAADC.unpend();
        unsafe { interrupt::SAADC.enable() };

        // calibrate
        r.events_calibratedone().write_value(0);
        r.intenset().write(|w| w.set_calibratedone(true));
        // Order is important
        atomic::compiler_fence(Ordering::SeqCst);
        r.tasks_calibrateoffset().write_value(1);

        loop {
            if r.events_calibratedone().read() != 0 {
                defmt::info!("SAADC calibration done");
                r.events_calibratedone().write_value(0);
                break;
            }
        }

        // Set up the DMA
        r.result().ptr().write_value(buf.0.as_mut_ptr() as u32);
        r.result()
            .maxcnt()
            .write(|w| w.set_maxcnt(buf.0.len() as _));

        // Reset and enable the start and end event
        r.events_end().write_value(0);
        r.intenset().write(|w| w.set_end(true));
        //r.intenset().write(|w| w.set_started(true));

        r.events_done().write_value(0);
        r.intenset().write(|w| w.set_done(true));

        r.intenset().write(|w| w.set_chlimitl(0, true));
        r.intenset().write(|w| w.set_chlimith(0, true));

        // Don't reorder the ADC start event before the previous writes. Hopefully self
        // wouldn't happen anyway.
        atomic::compiler_fence(Ordering::SeqCst);

        r.tasks_start().write_value(1);
        // triggered once to start continuous sampling.
        // See 6.23.2.2 in spec
        r.tasks_sample().write_value(1);

        Self {
            _p: saadc,
            debounce_delay_us: 10_000,
            last_change_time_us: uptime_us(),
            val: 0, //Self::read_reg(),
            buf,
        }
    }

    pub fn read(&self) -> i16 {
        self.val
    }

    fn update(&mut self, current_time_us: u64) {
        // let r = Self::regs();
        // r.tasks_sample().write_value(1);

        if current_time_us.wrapping_sub(self.last_change_time_us) >= self.debounce_delay_us {
            self.val = Self::read_reg(self);
        }
    }

    fn regs() -> hal::pac::saadc::Saadc {
        hal::pac::SAADC
    }

    fn read_reg(&self) -> i16 {
        // defmt::println!("Buffer: {}", defmt::Debug2Format(self.buf));
        self.buf.0[0]
    }
}

const SAADC_BUFFER_SIZE: usize = 1;
#[repr(C, align(4))]
#[derive(Debug, Copy, Clone)]
struct SaadcBuffer([i16; SAADC_BUFFER_SIZE]);

hal::bind_interrupts!(struct Irqs {
    SAADC => SaadcInterruptHandler;
});

pub struct SaadcInterruptHandler {
    _private: (),
}

impl typelevel::Handler<typelevel::SAADC> for SaadcInterruptHandler {
    unsafe fn on_interrupt() {
        let r = hal::pac::SAADC;

        if r.events_calibratedone().read() != 0 {
            r.intenclr().write(|w| w.set_calibratedone(true));
        }

        if r.events_end().read() != 0 {
            defmt::info!("SAADC buffer filled");
            r.intenclr().write(|w| w.set_end(true));
        }

        if r.events_started().read() != 0 {
            //defmt::info!("SAADC started");
            //r.intenclr().write(|w| w.set_started(true));
        }

        if r.events_ch(0).limitl().read().limitl() {
            defmt::info!("SAADC input below limit");
            r.intenclr().write(|w| w.set_chlimitl(0, true));
        }

        if r.events_ch(0).limith().read().limith() {
            defmt::info!("SAADC input above limit");
            r.intenclr().write(|w| w.set_chlimith(0, true));
        }

        if r.events_done().read() == 1 {
            defmt::info!("Sample taken");
            r.intenclr().write(|w| w.set_done(true));
        }
    }
}

/// Needed plain digital outputs on the board
pub struct DigitalOutputs {
    /// Pin p1.01
    pub p1_01: Output<'static>,
    /// Pin p1.02
    pub p1_02: Output<'static>,
    /// Pin p1.03
    pub p1_03: Output<'static>,
    /// Pin p1.04
    pub p1_04: Output<'static>,
}

/// Debounced input to flatten input noise
pub struct DebouncedInput<'i> {
    /// The input that is debounced
    inner: Input<'i>,
    prev_level: Level,
    /// The debounced input level
    level: Level,
    /// The last ms system time when the debounced input changed
    last_change_time_us: u64,
    /// The ms delay any change of the inner input is ignored after input level change
    debounce_delay_us: u64,
    toggled: bool,
    toggled_falling: bool,
    toggled_rising: bool,
}

impl<'i> DebouncedInput<'i> {
    /// Create a new debounced input with a debounce delay of 30ms
    pub fn new(input: Input<'i>) -> Self {
        let level = input.get_level();
        Self {
            inner: input,
            prev_level: level,
            level,
            last_change_time_us: 0,
            debounce_delay_us: 30_000, // 30ms
            toggled: false,
            toggled_falling: false,
            toggled_rising: false,
        }
    }

    /// Create a new debounced input with custom debounce delay given in ms
    pub fn with(input: Input<'i>, debounce_delay_us: u64) -> Self {
        let mut i = Self::new(input);
        i.debounce_delay_us = debounce_delay_us;
        i
    }

    /// Get the debounced input level
    pub fn get_level(&self) -> Level {
        self.level
    }

    /// Returns 'true' if the debounced input is `High`
    pub fn is_high(&self) -> bool {
        self.get_level() == Level::High
    }

    /// Returns 'true' if the debounced input is `low`
    pub fn is_low(&self) -> bool {
        self.get_level() == Level::Low
    }

    /// Returns 'true' if the debounced input level changed
    pub fn toggled(&mut self) -> bool {
        self.toggled
    }

    /// Returns 'true' if the debounced input level changed from `Low` to `High`
    pub fn toggled_rising(&mut self) -> bool {
        self.toggled_rising
    }

    /// Returns 'true' if the debounced input level changed from `High` to `Low`
    pub fn toggled_falling(&mut self) -> bool {
        self.toggled_falling
    }

    fn update_level(&mut self, current_time_us: u64) {
        // Resetting so toggle states are 'true' only for one loop iteration
        self.toggled = false;
        self.toggled_falling = false;
        self.toggled_rising = false;

        if current_time_us.wrapping_sub(self.last_change_time_us) >= self.debounce_delay_us {
            self.last_change_time_us = current_time_us;
            self.prev_level = self.level;
            self.level = self.inner.get_level();

            self.toggled = self.prev_level != self.level;
            self.toggled_falling = self.prev_level == Level::High && self.level == Level::Low;
            self.toggled_rising = self.prev_level == Level::Low && self.level == Level::High;
        }
    }
}

/// Needed plain digital inputs on the board
pub struct DigitalInputs {
    /// Pin p1.05
    pub p1_05: DebouncedInput<'static>,
    /// Pin p1.06
    pub p1_06: DebouncedInput<'static>,
    /// Pin p1.07
    pub p1_07: DebouncedInput<'static>,
    /// Pin p1.08
    pub p1_08: DebouncedInput<'static>,
}

/// All LEDs on the board
pub struct Leds {
    /// LED1: pin P0.13, green LED
    pub _1: Led,
    /// LED2: pin P0.14, green LED
    pub _2: Led,
    /// LED3: pin P0.15, green LED
    pub _3: Led,
    /// LED4: pin P0.16, green LED
    pub _4: Led,
}

/// A single LED
pub struct Led {
    port: Port,
    pin: u8,
    inner: Output<'static>,
}

impl defmt::Format for Led {
    fn format(&self, fmt: defmt::Formatter) {
        if self.is_on() {
            defmt::write!(fmt, "ON")
        } else {
            defmt::write!(fmt, "OFF")
        }
    }
}

impl Led {
    /// Turns on the LED
    pub fn on(&mut self) {
        defmt::trace!(
            "setting P{}.{} low (LED on)",
            if self.port == Port::Port1 { '1' } else { '0' },
            self.pin
        );

        self.inner.set_low()
    }

    /// Turns off the LED
    pub fn off(&mut self) {
        defmt::trace!(
            "setting P{}.{} high (LED off)",
            if self.port == Port::Port1 { '1' } else { '0' },
            self.pin
        );

        self.inner.set_high()
    }

    /// Returns `true` if the LED is in the OFF state
    pub fn is_off(&self) -> bool {
        self.inner.is_set_high()
    }

    /// Returns `true` if the LED is in the ON state
    pub fn is_on(&self) -> bool {
        !self.is_off()
    }

    /// Toggles the state (on/off) of the LED
    pub fn toggle(&mut self) {
        if self.is_off() {
            self.on();
        } else {
            self.off()
        }
    }
}

/// All Buttons on the board
pub struct Buttons {
    /// Button1: pin P0.11
    pub _1: Button,
    /// Button2: pin P0.12
    pub _2: Button,
    /// Button3: pin P0.24
    pub _3: Button,
    /// Button4: pin P0.25
    pub _4: Button,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwitchState {
    On,
    Off,
}

impl SwitchState {
    fn toggle(&mut self) {
        match self {
            SwitchState::On => *self = SwitchState::Off,
            SwitchState::Off => *self = SwitchState::On,
        }
    }
}

/// A single Button
pub struct Button {
    inner: DebouncedInput<'static>,
    state: SwitchState,
}

impl Button {
    pub fn new(input: DebouncedInput<'static>) -> Self {
        Self {
            inner: input,
            state: SwitchState::Off,
        }
    }

    /// Is the button pressed
    pub fn is_pressed(&self) -> bool {
        self.inner.is_low()
    }

    /// Has the button been toggled
    pub fn toggled(&mut self) -> bool {
        self.inner.toggled()
    }

    pub fn switched_on(&self) -> bool {
        self.state == SwitchState::On
    }

    pub fn switched_off(&self) -> bool {
        self.state == SwitchState::Off
    }

    fn update_state(&mut self, current_time_us: u64) {
        self.inner.update_level(current_time_us);

        // Button is `Low` = pressed, so `High` -> `Low` means 'non-pressed' -> 'pressed'
        if self.inner.toggled_falling() {
            self.state.toggle();
        }
    }
}

/// A timer for creating blocking delays
pub struct Timer(hal::timer::Timer<'static>);

impl DelayNs for Timer {
    fn delay_ns(&mut self, ns: u32) {
        if ns == 0 {
            return;
        }
        self.0.stop();
        self.0.clear();
        // Write cycle count in microseconds for 1 MHz timer.
        self.0.cc(0).write(ns / 1_000);
        self.0.start();
        while !self.reset_if_finished() {
            spin_loop();
        }
    }
}

impl Timer {
    /// Create a new timer instance which can be used for blocking delays.
    pub fn new<T: hal::timer::Instance>(peri: Peri<'static, T>) -> Self {
        let timer = hal::timer::Timer::new(peri);
        timer.set_frequency(hal::timer::Frequency::F1MHz);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();
        Self(timer)
    }

    /// Start the timer with the given microsecond duration.
    pub fn start(&mut self, microseconds: u32) {
        self.0.stop();
        self.0.clear();
        self.0.cc(0).write(microseconds);
        self.0.start();
    }

    /// If the timer has finished, resets it and returns true.
    ///
    /// Returns false if the timer is still running.
    pub fn reset_if_finished(&mut self) -> bool {
        if !self.0.cc(0).event_compare().is_triggered() {
            // EVENTS_COMPARE has not been triggered yet
            return false;
        }

        self.0.cc(0).clear_events();

        true
    }

    /// Wait for the specified duration.
    pub fn wait(&mut self, duration: Duration) {
        defmt::trace!("blocking for {:?} ...", duration);

        // 1 cycle = 1 microsecond
        let subsec_micros = duration.subsec_micros();
        if subsec_micros != 0 {
            self.delay_us(subsec_micros);
        }

        let mut millis = duration.as_secs() * 1000;
        if millis == 0 {
            return;
        }

        while millis > u32::MAX as u64 {
            self.delay_ms(u32::MAX);
            millis -= u32::MAX as u64;
        }
        self.delay_ms(millis as u32);

        defmt::trace!("... DONE");
    }
}

/// The ways that initialisation can fail
#[derive(Debug, Copy, Clone, defmt::Format)]
pub enum Error {
    /// You tried to initialise the board twice
    DoubleInit = 1,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::DoubleInit => write!(f, "You tried to initialise the board twice"),
        }
    }
}

impl core::error::Error for Error {}

// Atomic flag to detect double initialization of the HAL.
static HAL_INIT: AtomicBool = AtomicBool::new(false);

impl Board {
    /// Initializes the board
    pub fn init() -> Result<Self, Error> {
        if HAL_INIT.swap(true, Ordering::Relaxed) {
            return Err(Error::DoubleInit);
        }

        let mut config = hal::config::Config::default();
        config.hfclk_source = hal::config::HfclkSource::ExternalXtal;
        config.lfclk_source = hal::config::LfclkSource::ExternalXtal;
        let periph = hal::init(config);

        // probe-rs puts us in blocking mode, so wait for blocking mode as a proxy
        // for waiting for probe-rs to connect.
        //
        // do this *after* clock set-up to avoid start-up issues

        // TODO: use feature flag to toggle code below
        // while !defmt_rtt::in_blocking_mode() {
        //     core::hint::spin_loop();
        // }

        // NOTE: this branch runs at most once

        // Section 6.22.2 p. 660 in nrf spec shows how to calculate the counter increment frequency
        // f_rtc [kHz] = 32.768 / (PRESCALER + 1)
        let mut rtc = hal::rtc::Rtc::new(periph.RTC0, 0).unwrap();
        // NOTE on unmasking the NVIC interrupt: Because this crate defines the `#[interrupt] fn RTC0`
        // interrupt handler, RTIC cannot manage that interrupt (trying to do so results in a linker
        // error). Thus it is the task of this crate to mask/unmask the interrupt in a safe manner.
        //
        // Because the RTC0 interrupt handler does *not* access static variables through a critical
        // section (that disables interrupts) this `unmask` operation cannot break critical sections
        // and thus won't lead to undefined behavior (e.g. torn reads/writes)
        //
        // the preceding `enable_conuter` method consumes the `rtc` value. This is a semantic move
        // of the RTC0 peripheral from this function (which can only be called at most once) to the
        // interrupt handler (where the peripheral is accessed without any synchronization
        // mechanism)
        rtc.enable_interrupt(hal::rtc::Interrupt::Overflow, true);
        rtc.enable();

        defmt::debug!("RTC started");

        let dig_out_p1_01 = Output::new(periph.P1_01, Level::Low, OutputDrive::Standard);
        let dig_out_p1_02 = Output::new(periph.P1_02, Level::Low, OutputDrive::Standard);
        let dig_out_p1_03 = Output::new(periph.P1_03, Level::Low, OutputDrive::Standard);
        let dig_out_p1_04 = Output::new(periph.P1_04, Level::Low, OutputDrive::Standard);

        let dig_in_p1_05 = DebouncedInput::new(Input::new(periph.P1_05, hal::gpio::Pull::Up));
        let dig_in_p1_06 = DebouncedInput::new(Input::new(periph.P1_06, hal::gpio::Pull::Up));
        let dig_in_p1_07 = DebouncedInput::new(Input::new(periph.P1_07, hal::gpio::Pull::Up));
        let dig_in_p1_08 = DebouncedInput::new(Input::new(periph.P1_08, hal::gpio::Pull::Up));

        let led1pin = Led {
            port: Port::Port0,
            pin: 13,
            inner: Output::new(periph.P0_13, Level::High, OutputDrive::Standard),
        };
        let led2pin = Led {
            port: Port::Port0,
            pin: 14,
            inner: Output::new(periph.P0_14, Level::High, OutputDrive::Standard),
        };
        let led3pin = Led {
            port: Port::Port0,
            pin: 15,
            inner: Output::new(periph.P0_15, Level::High, OutputDrive::Standard),
        };
        let led4pin = Led {
            port: Port::Port0,
            pin: 16,
            inner: Output::new(periph.P0_16, Level::High, OutputDrive::Standard),
        };

        defmt::debug!("I/O pins have been configured for digital output");

        // NOTE pin goes low when button is pressed
        let button1pin = Button::new(DebouncedInput::new(Input::new(
            periph.P0_11,
            hal::gpio::Pull::Up,
        )));
        let button2pin = Button::new(DebouncedInput::new(Input::new(
            periph.P0_12,
            hal::gpio::Pull::Up,
        )));
        let button3pin = Button::new(DebouncedInput::new(Input::new(
            periph.P0_24,
            hal::gpio::Pull::Up,
        )));
        let button4pin = Button::new(DebouncedInput::new(Input::new(
            periph.P0_25,
            hal::gpio::Pull::Up,
        )));

        let timer = Timer::new(periph.TIMER0);

        Ok(Self {
            leds: Leds {
                _1: led1pin,
                _2: led2pin,
                _3: led3pin,
                _4: led4pin,
            },
            buttons: Buttons {
                _1: button1pin,
                _2: button2pin,
                _3: button3pin,
                _4: button4pin,
            },
            dig_out: DigitalOutputs {
                p1_01: dig_out_p1_01,
                p1_02: dig_out_p1_02,
                p1_03: dig_out_p1_03,
                p1_04: dig_out_p1_04,
            },
            dig_in: DigitalInputs {
                p1_05: dig_in_p1_05,
                p1_06: dig_in_p1_06,
                p1_07: dig_in_p1_07,
                p1_08: dig_in_p1_08,
            },
            timer,
            analog_in: AnalogInput::new(periph.SAADC),
        })
    }

    /// Update debounced input states.
    /// This ensures that all inputs are updated at roughly the same time.
    pub fn update(&mut self) {
        let current_time_us = uptime_us();

        self.dig_in.p1_05.update_level(current_time_us);
        self.dig_in.p1_06.update_level(current_time_us);
        self.dig_in.p1_07.update_level(current_time_us);
        self.dig_in.p1_08.update_level(current_time_us);

        self.buttons._1.update_state(current_time_us);
        self.buttons._2.update_state(current_time_us);
        self.buttons._3.update_state(current_time_us);
        self.buttons._4.update_state(current_time_us);

        self.analog_in.update(current_time_us);
    }

    pub fn sys_time(&self) -> Duration {
        uptime()
    }

    pub fn print_io(&self) {
        defmt::info!("Out p1.01 is '{}'", &self.dig_out.p1_01.get_output_level());
        defmt::info!("Out p1.02 is '{}'", &self.dig_out.p1_02.get_output_level());
        defmt::info!("Out p1.03 is '{}'", &self.dig_out.p1_03.get_output_level());
        defmt::info!("Out p1.04 is '{}'", &self.dig_out.p1_04.get_output_level());

        defmt::info!("Inp p1.05 is '{}'", &self.dig_in.p1_05.get_level());
        defmt::info!("Inp p1.06 is '{}'", &self.dig_in.p1_06.get_level());
        defmt::info!("Inp p1.07 is '{}'", &self.dig_in.p1_07.get_level());
        defmt::info!("Inp p1.08 is '{}'", &self.dig_in.p1_08.get_level());

        defmt::info!("LED1 is '{}'", &self.leds._1);
        defmt::info!("LED2 is '{}'", &self.leds._2);
        defmt::info!("LED3 is '{}'", &self.leds._3);
        defmt::info!("LED4 is '{}'", &self.leds._4);

        defmt::info!("Analog input p0.03 '{}'", self.analog_in.read());
    }
}

// Counter of OVERFLOW events -- an OVERFLOW occurs every (1<<24) ticks
static OVERFLOWS: AtomicU32 = AtomicU32::new(0);

// NOTE this will run at the highest priority, higher priority than RTIC tasks
#[interrupt]
fn RTC0() {
    OVERFLOWS.fetch_add(1, Ordering::Release);
    let rtc = hal::pac::RTC0;
    // clear the EVENT register
    rtc.events_ovrflw().write_value(0);
}

/// Exits the application successfully when the program is executed through the
/// `probe-rs` Cargo runner
pub fn exit() -> ! {
    unsafe {
        // turn off the USB D+ pull-up before pausing the device with a breakpoint
        // this disconnects the nRF device from the USB host so the USB host won't attempt further
        // USB communication (and see an unresponsive device).
        const USBD_USBPULLUP: *mut u32 = 0x4002_7504 as *mut u32;
        USBD_USBPULLUP.write_volatile(0)
    }
    defmt::println!("`nrf_hal::exit()` called; exiting ...");
    // force any pending memory operation to complete before the instruction that follows
    atomic::compiler_fence(Ordering::SeqCst);
    loop {
        debug::exit(debug::ExitStatus::Ok(()))
    }
}

/// Exits the application with a failure when the program is executed through
/// the `probe-rs` Cargo runner
pub fn fail() -> ! {
    unsafe {
        // turn off the USB D+ pull-up before pausing the device with a breakpoint
        // this disconnects the nRF device from the USB host so the USB host won't attempt further
        // USB communication (and see an unresponsive device).
        const USBD_USBPULLUP: *mut u32 = 0x4002_7504 as *mut u32;
        USBD_USBPULLUP.write_volatile(0)
    }
    defmt::println!("`nrf_hal::fail()` called; exiting ...");
    // force any pending memory operation to complete before the instruction that follows
    atomic::compiler_fence(Ordering::SeqCst);
    loop {
        debug::exit(debug::ExitStatus::Err(()))
    }
}

/// Returns the time elapsed since the call to the `Board::init` function
///
/// The time is in 32,768 Hz units (i.e. 32768 = 1 second)
///
/// Calling this function before calling `Board::init` will return a value of `0` nanoseconds.
pub fn uptime_ticks() -> u64 {
    // here we are going to perform a 64-bit read of the number of ticks elapsed
    //
    // a 64-bit load operation cannot performed in a single instruction so the operation can be
    // preempted by the RTC0 interrupt handler (which increases the OVERFLOWS counter)
    //
    // the loop below will load both the lower and upper parts of the 64-bit value while preventing
    // the issue of mixing a low value with an "old" high value -- note that, due to interrupts, an
    // arbitrary amount of time may elapse between the `hi1` load and the `low` load

    // # Safety
    // Concurrent access to this field within the RTC is acceptable.
    let rtc_counter = hal::pac::RTC0.counter();

    loop {
        // NOTE volatile is used to order these load operations among themselves
        let hi1 = OVERFLOWS.load(Ordering::Acquire);
        let low = rtc_counter.read().counter();
        let hi2 = OVERFLOWS.load(Ordering::Relaxed);

        if hi1 == hi2 {
            // << 24, because RTC is a 24-bit counter according to spec section 6.22 p. 660
            break u64::from(low) | (u64::from(hi1) << 24);
        }
    }
}

/// Returns the time elapsed since the call to the `Board::init` function
///
/// The clock that is read to compute this value has a resolution of 30 microseconds.
///
/// Calling this function before calling `Board::init` will return a value of `0` nanoseconds.
pub fn uptime() -> Duration {
    // We have a time in 32,768 Hz units.
    let mut ticks = uptime_ticks();

    // turn it into 32_768_000_000 units
    ticks = ticks.wrapping_mul(1_000_000);
    // turn it into microsecond units
    ticks >>= 15;
    // turn it into nanosecond units
    ticks = ticks.wrapping_mul(1_000);

    // NB: 64-bit nanoseconds handles around 584 years.

    let secs = ticks / 1_000_000_000;
    let nanos = ticks % 1_000_000_000;

    Duration::new(secs, nanos as u32)
}

/// Returns the time elapsed since the call to the `Board::init` function, in microseconds.
///
/// The clock that is read to compute this value has a resolution of 30 microseconds.
/// See section 6.22.2 p. 661 in NRF spec for prescaler = 0 (set in `Board::init`)
///
/// Calling this function before calling `Board::init` will return a value of `0` nanoseconds.
pub fn uptime_us() -> u64 {
    // We have a time in 32,768 Hz units.
    let mut ticks = uptime_ticks();

    // turn it into 32_768_000_000 units
    ticks = ticks.wrapping_mul(1_000_000);
    // turn it into microsecond units
    ticks >>= 15;

    ticks
}

/// Returns the least-significant bits of the device identifier
pub fn deviceid0() -> u32 {
    hal::pac::FICR.deviceid(0).read()
}

/// Returns the most-significant bits of the device identifier
pub fn deviceid1() -> u32 {
    hal::pac::FICR.deviceid(1).read()
}
