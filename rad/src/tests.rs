use mantra_macros::req_test;

use crate::{
    hal::{LedIndicator, MockHal, OutputState},
    operation_conditions_fulfilled, Rad, RadError, RadMode,
};

#[req_test("rad.sw.operation.pre-condition")]
#[test]
fn operation_condition_fulfilled() {
    let mut mock = MockHal::default();

    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);

    assert!(
        operation_conditions_fulfilled(&mock).is_ok(),
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

#[req_test("rad.sw.idle")]
#[test]
fn default_idle() {
    let rad = Rad::init();

    assert_eq!(rad.mode, RadMode::Idle, "RAD must start in idle mode");
}

#[req_test("rad.sw.operation.pre-condition")]
#[test]
fn request_start_on_valid_pre_condition() {
    let mut mock = MockHal::default();

    let expect_on_f = |indicator: &LedIndicator| indicator == &LedIndicator::On;

    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);
    mock.expect_start_requested().return_const(true);
    mock.expect_radiation_active().return_const(false);
    mock.expect_set_mode_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_confirmation_switch_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_entrance_door_indicator()
        .withf(expect_on_f)
        .return_const(());
    // Radiation has not been *mocked* to be active yet
    mock.expect_radiation_output_state()
        .return_const(OutputState::Off);
    mock.expect_set_radiation_output_indicator()
        .withf(|indicator| indicator == &LedIndicator::Off)
        .return_const(());

    let mut rad = Rad::init();

    rad.update(&mut mock);

    assert_eq!(
        rad.mode,
        RadMode::Operation,
        "RAD should have switched into operation"
    );
}

#[req_test("rad.sw.operation.stop", "rad.sw.operation.post-condition")]
#[test]
fn request_operation_stop() {
    let expect_on_f = |indicator: &LedIndicator| indicator == &LedIndicator::On;

    let mut mock = MockHal::default();
    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);
    mock.expect_start_requested().return_const(true);
    mock.expect_radiation_active().return_const(false);
    mock.expect_set_mode_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_confirmation_switch_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_entrance_door_indicator()
        .withf(expect_on_f)
        .return_const(());
    // Radiation has not been *mocked* to be active yet
    mock.expect_radiation_output_state()
        .return_const(OutputState::Off);
    mock.expect_set_radiation_output_indicator()
        .withf(|indicator| indicator == &LedIndicator::Off)
        .return_const(());

    let mut rad = Rad::init();

    rad.update(&mut mock); // get into operation

    assert_eq!(
        rad.mode,
        RadMode::Operation,
        "RAD should have switched into operation"
    );

    let mut mock = MockHal::default();
    mock.expect_stop_requested().return_const(false);
    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);

    mock.expect_set_mode_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_confirmation_switch_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_entrance_door_indicator()
        .withf(expect_on_f)
        .return_const(());
    // Radiation being activated
    mock.expect_radiation_active().return_const(false);
    mock.expect_start_radiation().once().return_const(());
    mock.expect_radiation_output_state()
        .return_const(OutputState::On);
    mock.expect_set_radiation_output_indicator()
        .withf(|indicator| indicator == &LedIndicator::On)
        .return_const(());

    rad.update(&mut mock); // set radiation active

    // Stopping
    let mut mock = MockHal::default();
    mock.expect_stop_requested().return_const(true);
    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);
    mock.expect_set_mode_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_confirmation_switch_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_entrance_door_indicator()
        .withf(expect_on_f)
        .return_const(());
    // We say that radiation is still active
    mock.expect_stop_radiation().return_const(());
    mock.expect_radiation_active().return_const(true);
    mock.expect_radiation_output_state()
        .return_const(OutputState::On);
    mock.expect_set_radiation_output_indicator()
        .withf(|indicator| indicator == &LedIndicator::On)
        .return_const(());

    rad.update(&mut mock); // stopping

    // radiation stopped -> idle

    let mut mock = MockHal::default();
    mock.expect_stop_requested().return_const(false); // stop already set
    mock.expect_entrance_door_closed().return_const(true);
    mock.expect_safe_environment_confirmed().return_const(true);
    mock.expect_set_mode_indicator()
        .withf(|indicator| indicator == &LedIndicator::Off)
        .return_const(());
    mock.expect_set_confirmation_switch_indicator()
        .withf(expect_on_f)
        .return_const(());
    mock.expect_set_entrance_door_indicator()
        .withf(expect_on_f)
        .return_const(());
    // We say that radiation is still active
    mock.expect_stop_radiation().return_const(());
    mock.expect_radiation_active().return_const(false);
    mock.expect_radiation_output_state()
        .return_const(OutputState::Off);
    mock.expect_set_radiation_output_indicator()
        .withf(|indicator| indicator == &LedIndicator::Off)
        .return_const(());

    rad.update(&mut mock);

    assert_eq!(
        rad.mode,
        RadMode::Idle,
        "RAD should have switched back into idle"
    );
}
