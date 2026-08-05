#![no_main]
#![no_std]

use sim::{self as _, Sim};

#[cortex_m_rt::entry]
fn main() -> ! {
    let sim = Sim::init();

    sim.print();

    loop {
        core::hint::spin_loop();
    }
}
