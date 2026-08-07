mantra:
    rm -rf target/mantra-report
    mantra collect
    mantra report --output-dir=target/mantra-report

base:
    just rad-unit-tests
    just rad-system-tests
    just mantra

[working-directory("sim")]
sim-manual:
    DEFMT_LOG=info cargo run --bin manual --no-default-features

[working-directory("sim")]
sim-start-stop:
    DEFMT_LOG=info cargo run --bin start_stop_flow --no-default-features

[working-directory("sim")]
sim-build-start-stop:
    DEFMT_LOG=info cargo build --bin start_stop_flow

[working-directory("sim")]
sim-build-invariant-check:
    DEFMT_LOG=info cargo build --bin invariant_check

[working-directory("sim")]
sim-invariant-check:
    DEFMT_LOG=info cargo run --bin invariant_check --no-default-features

[working-directory("sim")]
sim-reset:
    DEFMT_LOG=info cargo run --bin reset

[working-directory("rad")]
rad-run:
    DEFMT_LOG=info cargo run --bin rad --features=hw-testing

[working-directory("rad")]
rad-build features='hw':
    DEFMT_LOG=info cargo build --bin rad --features={{ features }}

profraw-file := justfile_directory() + "/target/nextest/default/raw-coverage/profdata-%p-%m.profraw"

rad-unit-tests:
    rm -rf target/nextest/default
    mkdir -p target/nextest/default/coverage/raw-coverage
    - RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="{{ profraw-file }}" cargo nextest run -p rad --lib --target=host-tuple --no-default-features
    grcov . -s . --binary-path ./target -t html -t cobertura-pretty --ignore-not-existing -o ./target/nextest/default/coverage/ --ignore='/**/*' --ignore='target/*'

export EMBSINTH_OUT_DIR := justfile_directory() + "/target"

[working-directory("system-tests")]
rad-system-tests:
    rm -rf $EMBSINTH_OUT_DIR/system-tests
    just rad-build hw-auto-testing
    just sim-build-start-stop
    just sim-build-invariant-check
    RUST_LOG=info cargo test --target=host-tuple 
    just post-process

post-process tests='system-tests':
    embsinth post-process --out $EMBSINTH_OUT_DIR/{{ tests }}/mantra_test_run.json --test-run-name {{ tests }} $EMBSINTH_OUT_DIR/

slides:
    typst compile --format pdf --font-path assets/fonts --root $PWD docs/slides.typ slides.pdf
