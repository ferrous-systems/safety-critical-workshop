use std::time::Duration;

use mantra_macros::{assert_req, req_verified};
use system_tests::{BINARY_DIR, init};

#[cfg(feature = "phase-one")]
#[req_verified("rad.sw.operation.invariant")]
#[embsinth::test]
fn invariant_check() {
    let (mut rad_connection, mut sim_connection) = init(&BINARY_DIR.join("invariant_check"));

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

    assert!(
        sim_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message.starts_with("Breaking RAD invariant")
            })
            .is_some(),
        "Invariant was not violated by simulation"
    );

    assert!(
        rad_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message.starts_with("Operation invariant violated")
            })
            .is_some(),
        "Invariant violation not detected"
    );

    assert!(
        sim_connection
            .search_msg_for(Duration::from_secs(5), |msg| {
                msg.message
                    .starts_with("RAD invariant flow done -> exiting")
            })
            .is_some(),
        "Invariant simulation did not succeed"
    );
}
