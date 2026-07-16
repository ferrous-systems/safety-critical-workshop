use mantra_macros::req;
#[cfg(test)]
use mockall::automock;

pub struct Hal {
    board: nrf_hal::Board,
}

#[cfg_attr(test, automock)]
impl Hal {
    pub fn init() -> Result<Self, nrf_hal::Error> {
        let board = nrf_hal::Board::init()?;

        Ok(Self { board })
    }

    #[req("rad.hw.start-stop-switch")]
    pub fn start_requested(&self) -> bool {
        self.board.dig_in.p1_05.is_low()
    }

    #[req("rad.hw.start-stop-switch")]
    pub fn stop_requested(&self) -> bool {
        self.board.dig_in.p1_05.is_high()
    }

    #[req("rad.hw.radiation-relay")]
    pub fn start_radiation(&mut self) {}

    #[req("rad.hw.radiation-relay")]
    pub fn stop_radiation(&mut self) {}

    pub fn radiation_active(&self) -> bool {
        // TODO: replace with analog input once board has support
        self.board.dig_in.p1_08.is_low()
    }

    pub fn entrance_door_closed(&self) -> bool {
        self.board.dig_in.p1_06.is_low()
    }

    pub fn safe_environment_confirmed(&self) -> bool {
        self.board.dig_in.p1_07.is_low()
    }
}
