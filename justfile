mantra:
    rm -rf target/mantra-report
    mantra collect
    mantra report --output-dir=target/mantra-report

[working-directory("sim")]
sim-manual:
    DEFMT_LOG=info cargo run --bin manual

[working-directory("sim")]
sim-start-stop:
    DEFMT_LOG=info cargo run --bin start_stop_flow

[working-directory("sim")]
sim-reset:
    DEFMT_LOG=info cargo run --bin reset

[working-directory("rad")]
rad-run:
    DEFMT_LOG=info cargo run --bin rad

[working-directory("rad")]
rad-unit-tests:
    cargo test --lib --target=host-tuple --no-default-features
