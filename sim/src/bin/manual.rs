#![no_main]
#![no_std]

use sim as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = nrf_hal::Board::init().unwrap();

    loop {
        if board.buttons._1.switched_on() {
            board.dig_out.p1_01.set_low();
            board.leds._1.on();
        } else {
            board.dig_out.p1_01.set_high();
            board.leds._1.off();
        }

        if board.buttons._2.switched_on() {
            board.dig_out.p1_02.set_low();
            board.leds._2.on();
        } else {
            board.dig_out.p1_02.set_high();
            board.leds._2.off();
        }

        if board.buttons._3.switched_on() {
            board.dig_out.p1_03.set_low();
            board.leds._3.on();
        } else {
            board.dig_out.p1_03.set_high();
            board.leds._3.off();
        }

        if board.buttons._4.switched_on() {
            board.dig_out.p1_04.set_low();
            board.leds._4.on();
        } else {
            board.dig_out.p1_04.set_high();
            board.leds._4.off();
        }

        board.update();
    }
}
