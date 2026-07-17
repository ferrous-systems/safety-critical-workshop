use mantra_macros::req_test;

use crate::{hal::MockHal, Rad, RadError};

#[req_test("rad.sw.operation.pre-condition")]
#[test]
fn operation_condition_fulfilled() {
    let mut mock = MockHal::default();

    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);

    assert!(
        let res = operation_conditions_fulfilled(&mock).is_ok(),
        "Pre-condition was mocked to be fulfilled"
    );
}

#[req_test("rad.sw.operation.pre-condition")]
#[test]
fn operation_condition_unfulfilled_door() {
    let mut mock = MockHal::default();

    mock.expect_entrance_door_closed().return_const(false);
    mock.expect_safe_environment_confirmed().return_const(true);

    let res = operation_conditions_fulfilled(&mock);

    assert!(res.is_err(), "Pre-condition was mocked to be unfulfilled");
    assert_eq!(res, Err(RadError::EntranceDoorOpen));
}

#[req_test("rad.sw.operation.pre-condition")]
#[test]
fn operation_condition_unfulfilled_confirmation() {
    let mut mock = MockHal::default();

    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(false);

    let res = operation_conditions_fulfilled(&mock);

    assert!(res.is_err(), "Pre-condition was mocked to be unfulfilled");
    assert_eq!(res, Err(RadError::MissingSafeEnvironmentConfirmation));
}
