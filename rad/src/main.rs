#![no_main]
#![no_std]

use cortex_m_rt::exception;
use mantra_macros::satisfy_req;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = satisfy_req!("rad.sw.hal" => nrf_hal::Board::init().unwrap());
    let mut rad = rad::Rad::init();

    //board.print_io();

    let mut last_print_time = board.sys_time();

    loop {
        board.update();
        rad.update(&mut board);

        if last_print_time.abs_diff(board.sys_time()).as_secs() > 1 {
            last_print_time = board.sys_time();
            board.print_io();
        }
    }
}

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

#[cfg(feature = "hw")]
defmt::timestamp!("{=u64:tus}", nrf_hal::uptime_us());
