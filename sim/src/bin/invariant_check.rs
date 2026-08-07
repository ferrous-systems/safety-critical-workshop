#![no_main]
#![no_std]

use core::time::Duration;

use sim::{self as _, RadMode, Sim, UPDATE_DELAY_MS};

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

    sim.wait(Duration::from_millis(UPDATE_DELAY_MS));

    defmt::info!("Breaking RAD invariant");
    sim.set_environment_confirmation(sim::OutputState::Off);
    sim.wait_update();

    // RAD stays in 'operation' until radiation is deactivated
    sim.set_radiation_state(sim::RadiationState::Deactive);
    sim.wait_update();

    assert_eq!(
        sim.actual_mode(),
        RadMode::Idle,
        "Violating the invariant must move the RAD back to 'idle'"
    );

    defmt::info!("RAD invariant flow done -> exiting");

    nrf_hal::exit()
}
