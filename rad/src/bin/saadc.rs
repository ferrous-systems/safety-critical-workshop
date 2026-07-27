#![no_main]
#![no_std]

use core::time::Duration;

use cortex_m_rt::exception;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = nrf_hal::Board::init().unwrap();

    let mut buf: [i16; 1] = [0; 1];

    loop {
        board.analog_in.read(&mut buf);
        defmt::info!("ADC reading: {}", buf[0]);
        board.timer.wait(Duration::from_millis(2000));
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
