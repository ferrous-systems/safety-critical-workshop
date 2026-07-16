#![no_main]
#![no_std]

use sim as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = nrf_hal::Board::init().unwrap();

    loop {
        board.update();

        if board.buttons._1.switched_on() {
            board.dig_out.p1_01.set_low();
            board.leds._1.on();
        } else {
            board.dig_out.p1_01.set_high();
            board.leds._1.off();
        }
    }
}
