use std::time::Duration;

use mantra_macros::{assert_req, req_verified};
use system_tests::{RAD_BINARY_FILEPATH, WORKSPACE_DIR, init, rad_probe};

#[req_verified("rad.sw.hal")]
#[embsinth::test]
fn hw_init() {
    let rad_probe = rad_probe();

    let mut rad_connection = rad_probe
        .flash_once_and_connect(WORKSPACE_DIR.join(RAD_BINARY_FILEPATH))
        .expect("Failed to flash rad binary");

    assert!(
        rad_connection
            .search_msg_for(Duration::from_secs(10), |msg| {
                msg.message.starts_with("Starting up *RAD*")
            })
            .is_some(),
        "RAD HW failed to initialize"
    );
}

#[req_verified("rad.sw.operation")]
#[embsinth::test]
fn start_stop_flow() {
    let (mut rad_connection, mut sim_connection) =
        init(&WORKSPACE_DIR.join("target/thumbv7em-none-eabihf/debug/start_stop_flow"));

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
}
