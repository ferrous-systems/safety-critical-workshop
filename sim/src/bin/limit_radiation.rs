#![no_main]
#![no_std]

use sim::{self as _, OutputState, RadMode, Sim};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    sim.update_until(
        |sim| sim.actual_mode() == RadMode::Idle,
        "RAD not in 'idle' after startup delay",
    );

    defmt::info!("Starting RAD operation");
    sim.rad_to_production();

    assert_eq!(
        sim.radiation_relay(),
        OutputState::On,
        "Radiation relay must be turned on at start"
    );

    defmt::info!("Waiting until radiation limit is hit");
    sim.update_until_with_timeout(
        |sim| {
            (sim.radiation_relay() == OutputState::Off)
                .then(|| sim.set_radiation_state(sim::RadiationState::Deactive))
                .is_some()
        },
        10,
        "RAD did not raise radiation limit",
    );

    defmt::info!("Radiation limit hit, waiting until radiation falls again");
    sim.wait_update();

    sim.update_until_with_timeout(
        |sim| {
            (sim.radiation_relay() == OutputState::On)
                .then(|| sim.set_radiation_state(sim::RadiationState::Active))
                .is_some()
        },
        10,
        "RAD did not restart radiation after 10s",
    );

    defmt::info!("Moving RAD to 'idle'");

    sim.rad_to_idle();

    defmt::info!("RAD limit-radiation done -> exiting");

    nrf_hal::exit()
}
