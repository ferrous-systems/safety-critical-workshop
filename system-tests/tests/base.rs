use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use embsinth::connection::Connection;
use mantra_macros::{assert_req, req_verified};

fn init(sim_path: &Path) -> (Connection, Connection) {
    let crate_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Tests must be run via cargo"));
    let workspace_dir = crate_dir.parent().expect("Parent path must exist");

    let rad_probe = embsinth::probe::ProbeId::with_serial_nr(0x1366, 0x1051, "001050272949")
        .attach_under_reset("nRF52840_xxAA")
        .expect("Failed to attach to rad target");
    let sim_probe = embsinth::probe::ProbeId::with_serial_nr(0x1366, 0x1051, "001050286871")
        .attach_under_reset("nRF52840_xxAA")
        .expect("Failed to attach to sim target");

    let mut rad_connection = rad_probe
        .flash_once_and_connect(workspace_dir.join("target/thumbv7em-none-eabihf/debug/rad"))
        .expect("Failed to flash rad binary");
    let mut sim_connection = sim_probe
        .flash_and_connect(sim_path)
        .expect("Failed flashing to sim target");

    (rad_connection, sim_connection)
}

#[embsinth::test]
#[req_verified("rad.sw.operation")]
fn integration_test() {
    let crate_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Tests must be run via cargo"));
    let workspace_dir = crate_dir.parent().expect("Parent path must exist");

    let (mut rad_connection, mut sim_connection) =
        init(&workspace_dir.join("target/thumbv7em-none-eabihf/debug/start_stop_flow"));

    assert_req!("rad.sw.operation.start" =>
        rad_connection
            .search_msg_for(Duration::from_secs(10), |msg| {
                msg.message.starts_with("Start requested")
            })
            .is_some(),
        "No start was requested"
    );

    assert_req!("rad.sw.operation.pre-condition" =>
        rad_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message.starts_with("Switching into 'operation' mode")
            })
            .is_some(),
        "Rad did not switch into 'operation' mode"
    );

    assert_req!("rad.sw.operation.stop" =>
        rad_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message.starts_with("Stop requested")
            })
            .is_some(),
        "No stop was requested"
    );

    assert_req!("rad.sw.operation.post-condition" =>
        rad_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message.starts_with("Switching into 'idle' mode")
            })
            .is_some(),
        "Rad did not go back to 'idle'"
    );

    assert_req!("rad.sw.operation" =>
        sim_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message
                    .starts_with("RAD start/stop flow done -> exiting")
            })
            .is_some(),
        "Start-stop simulation did not succeed"
    );

    // rad_connection.close();
    // sim_connection.close();
}
