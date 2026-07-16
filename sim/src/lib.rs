#![no_std]

use cortex_m_rt::exception;

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

defmt::timestamp!("{=u64:tus}", nrf_hal::uptime_us());
