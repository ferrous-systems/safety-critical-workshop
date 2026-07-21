#![no_main]
#![no_std]

use core::time::Duration;

use sim::{self as _, Sim, UPDATE_DELAY_MS};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    defmt::info!("Starting RAD operation");
    sim.rad_to_production();

    sim.wait(Duration::from_millis(UPDATE_DELAY_MS));

    defmt::info!("Stopping RAD operation");
    sim.rad_to_idle();

    defmt::info!("RAD start/stop flow done -> exiting");

    nrf_hal::exit();
}
