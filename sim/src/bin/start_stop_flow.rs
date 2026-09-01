#![no_main]
#![no_std]

use sim::{self as _, RadMode, Sim, UPDATE_DELAY};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    sim.update_until(
        |sim| sim.actual_mode() == RadMode::Idle,
        "RAD not in 'idle' after startup delay",
    );

    defmt::info!("Starting RAD operation");
    sim.rad_to_production();

    sim.wait(UPDATE_DELAY);

    defmt::info!("Stopping RAD operation");
    sim.rad_to_idle();

    defmt::info!("RAD start/stop flow done -> exiting");

    nrf_bsp::exit()
}
