#![no_main]
#![no_std]

use core::ops::ControlFlow;

use sim::{self as _, Sim};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut sim = Sim::init();

    let mut last_printed = sim.sys_time();

    sim.update_loop(|sim| {
        if sim.board().buttons._1.switched_on() {
            sim.set_start_stop(sim::StartStopState::Start);
            sim.board().leds._1.on();
        } else {
            sim.set_start_stop(sim::StartStopState::Stop);
            sim.board().leds._1.off();
        }

        if sim.board().buttons._2.switched_on() {
            sim.set_door_sensor(sim::OutputState::On);
            sim.board().leds._2.on();
        } else {
            sim.set_door_sensor(sim::OutputState::Off);
            sim.board().leds._2.off();
        }

        if sim.board().buttons._3.switched_on() {
            sim.set_environment_confirmation(sim::OutputState::On);
            sim.board().leds._3.on();
        } else {
            sim.set_environment_confirmation(sim::OutputState::Off);
            sim.board().leds._3.off();
        }

        if sim.board().buttons._4.switched_on() {
            sim.set_radiation_state(sim::RadiationState::Active);
            sim.board().leds._4.on();
        } else {
            sim.set_radiation_state(sim::RadiationState::Deactive);
            sim.board().leds._4.off();
        }

        let curr_time = sim.board().sys_time();
        if last_printed.abs_diff(curr_time).as_secs() > 2 {
            last_printed = curr_time;

            sim.print();
        }

        ControlFlow::Continue(())
    })
}
