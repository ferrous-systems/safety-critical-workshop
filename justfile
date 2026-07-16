mantra:
    mantra collect
    mantra report --output-dir=target/mantra-report

[working-directory("sim")]
sim-manual:
    DEFMT_LOG=info cargo run --bin manual

[working-directory("rad")]
rad-run:
    DEFMT_LOG=info cargo run --bin rad
