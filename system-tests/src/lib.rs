use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use embsinth::{connection::Connection, probe::AttachedProbe};

pub static WORKSPACE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Tests must be run via cargo"));
    let workspace_dir = crate_dir.parent().expect("Parent path must exist");
    workspace_dir.to_path_buf()
});

pub const RAD_BINARY_FILEPATH: &str = "target/thumbv7em-none-eabihf/debug/rad";

pub fn rad_probe() -> AttachedProbe {
    embsinth::probe::ProbeId::with_serial_nr(0x1366, 0x1051, "001050272949")
        .attach_under_reset("nRF52840_xxAA")
        .expect("Failed to attach to rad target")
}

pub fn init(sim_path: &Path) -> (Connection, Connection) {
    let rad_probe = rad_probe();
    let sim_probe = embsinth::probe::ProbeId::with_serial_nr(0x1366, 0x1051, "001050286871")
        .attach_under_reset("nRF52840_xxAA")
        .expect("Failed to attach to sim target");

    let mut rad_connection = rad_probe
        .flash_once_and_connect(WORKSPACE_DIR.join(RAD_BINARY_FILEPATH))
        .expect("Failed to flash rad binary");
    let mut sim_connection = sim_probe
        .flash_and_connect(sim_path)
        .expect("Failed flashing to sim target");

    (rad_connection, sim_connection)
}
