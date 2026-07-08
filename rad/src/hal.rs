#[cfg(test)]
use mockall::automock;

pub struct Hal {}

#[cfg_attr(test, automock)]
impl Hal {
    pub fn init() -> Self {
        Self {}
    }

    pub fn start_requested(&self) -> bool {
        todo!()
    }

    pub fn stop_requested(&self) -> bool {
        todo!()
    }

    pub fn start_radiation(&mut self) {}

    pub fn stop_radiation(&mut self) {}

    pub fn radiation_active(&self) -> bool {
        todo!()
    }

    pub fn entrance_door_closed(&self) -> bool {
        todo!()
    }

    pub fn safe_environment_confirmed(&self) -> bool {
        todo!()
    }
}
