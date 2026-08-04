#![cfg_attr(feature = "hw", no_main)]
#![no_std]

#[cfg(feature = "hw")]
use core::time::Duration;

#[cfg(feature = "hw")]
use cortex_m_rt::exception;

#[cfg(feature = "hw")]
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = nrf_hal::Board::init().unwrap();
    let mut rad = rad::Rad::init();

    board.print_io();

    loop {
        board.update();
        rad.update(&mut board);

        board.print_io();
        board.timer.wait(Duration::from_millis(2000));
    }
}

/// Our custom panic handler.
#[cfg(feature = "hw")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    nrf_hal::fail();
}

/// The default HardFault handler just spins, so replace it.
#[cfg(feature = "hw")]
#[exception]
unsafe fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    defmt::error!("HardFault!");
    nrf_hal::fail();
}

// this prevents the panic message being printed *twice* when `defmt::panic!` is invoked
#[cfg(feature = "hw")]
#[defmt::panic_handler]
fn defmt_panic() -> ! {
    nrf_hal::fail();
}

#[cfg(feature = "hw")]
defmt::timestamp!("{=u64:tus}", nrf_hal::uptime_us());

#[cfg(not(feature = "hw"))]
fn main() {}
