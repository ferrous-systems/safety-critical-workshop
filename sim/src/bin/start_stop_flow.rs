#![no_main]
#![no_std]

use core::{ops::ControlFlow, time::Duration};

use sim::{self as _, RadMode, Sim, UPDATE_DELAY_MS};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    let start_time = sim.sys_time();
    sim.update_loop(|sim| {
        if sim.actual_mode() == RadMode::Idle {
            return ControlFlow::Break(());
        } else if start_time.abs_diff(sim.sys_time()).as_secs() > 5 {
            panic!("RAD not in 'idle' after startup delay");
        }
        ControlFlow::Continue(())
    });

    defmt::info!("Starting RAD operation");
    sim.rad_to_production();

    sim.wait(Duration::from_millis(UPDATE_DELAY_MS));

    defmt::info!("Stopping RAD operation");
    sim.rad_to_idle();

    defmt::info!("RAD start/stop flow done -> exiting");

    nrf_hal::exit();
}
