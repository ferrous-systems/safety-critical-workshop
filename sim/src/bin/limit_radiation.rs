#![no_main]
#![no_std]

use sim::{self as _, OutputState, RadMode, Sim};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    let start_time = sim.sys_time();
    loop {
        if sim.actual_mode() == RadMode::Idle {
            break;
        } else if start_time.abs_diff(sim.sys_time()).as_secs() > 5 {
            panic!("RAD not in 'idle' after startup delay");
        }
        sim.update();
    }

    defmt::info!("Starting RAD operation");
    sim.rad_to_production();

    assert_eq!(
        sim.radiation_relay(),
        OutputState::On,
        "Radiation relay must be turned on at start"
    );

    defmt::info!("Waiting until radiation limit is hit");
    let start_time = sim.sys_time();
    loop {
        if sim.radiation_relay() == OutputState::Off {
            sim.set_radiation_state(sim::RadiationState::Deactive);
            break;
        } else if start_time.abs_diff(sim.sys_time()).as_secs() > 10 {
            panic!("RAD did not raise radiation limit after 10s");
        }

        sim.wait_update();
    }

    defmt::info!("Radiation limit hit, waiting until radiation falls again");
    sim.wait_update();

    let start_time = sim.sys_time();
    loop {
        if sim.radiation_relay() == OutputState::On {
            sim.set_radiation_state(sim::RadiationState::Active);
            break;
        } else if start_time.abs_diff(sim.sys_time()).as_secs() > 10 {
            panic!("RAD did not restart radiation after 10s");
        }

        sim.wait_update();
    }

    defmt::info!("Moving RAD to 'idle'");

    sim.rad_to_idle();

    defmt::info!("RAD limit-radiation done -> exiting");

    nrf_hal::exit()
}
