#![cfg_attr(feature = "hw", no_main)]
#![no_std]

#[cfg(feature = "hw")]
use cortex_m_rt::exception;
#[cfg(feature = "hw")]
use mantra_macros::satisfy_req;

#[cfg(feature = "hw")]
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = satisfy_req!("rad.sw.hal" => nrf_bsp::Board::init().unwrap());
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

#[cfg(feature = "hw")]
/// Our custom panic handler.
#[cfg(feature = "hw")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    nrf_bsp::fail();
}

#[cfg(feature = "hw")]
/// The default HardFault handler just spins, so replace it.
#[cfg(feature = "hw")]
#[exception]
unsafe fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    defmt::error!("HardFault!");
    nrf_bsp::fail()
}

#[cfg(feature = "hw")]
// this prevents the panic message being printed *twice* when `defmt::panic!` is invoked
#[cfg(feature = "hw")]
#[defmt::panic_handler]
fn defmt_panic() -> ! {
    nrf_bsp::fail();
}

#[cfg(feature = "hw")]
defmt::timestamp!("{=u64:tus}", nrf_bsp::uptime_us());

#[cfg(not(feature = "hw"))]
fn main() {}
