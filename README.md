# safety-critical-workshop

This repository contains the accompanying demo for the "Safety-Critical Rust Development with Ferrocene" workshop.

## Prerequisites

**The following tools must be installed:**
- Install [criticalup](https://criticalup.ferrocene.dev/install.html)
- Docker installed and usable (see OS specific instructions below)
- **Optional:** Rust toolchain via [rustup](https://rust-lang.org/tools/install/).  
  See [Ferrocene Setup](#ferrocene-setup) on how to link Ferrocene with rustup.
- `cargo install mantra --locked`
- `cargo install cargo-nextest --locked`
- `cargo install grcov --locked`
- `cargo install just --locked`
- `cargo install embsinth --locked`

### macOS

To run docker container on macOS, you may install [lima](https://lima-vm.io).
Follow [their documentation](https://lima-vm.io/docs/examples/containers/docker/) to use it as alias for docker commands:

```sh
limactl start --mount-writable template:docker
export DOCKER_HOST=$(limactl list docker --format 'unix://{{.Dir}}/sock/docker.sock')
# To verify if it worked, run:
docker run -d --name nginx -p 127.0.0.1:8080:80 nginx:alpine
```

**Note:** `limactl start --mount-writable template:docker` must only be run the first time. Afterwards, run `limactl start docker`.

**WARN:** Setting `--mount-writable` makes the home directory writable from the container.
This is needed to get raw LLVM coverage data during unit testing, but may pose security risks if other docker container are run.

## Facade Target Setup

Ferrocene's [Facade targets](https://public-docs.ferrocene.dev/main/user-manual/rustc/testing-facades.html)
allow to run regular Rust unit tests in an emulator of the CPU architecture of the actual target.

### QEMU Docker Container

For convenience, the Dockerfile in this repository provides the needed `qemu-arm-static` binary to run `thumbv7em` binaries.

**To build the image locally, run:**

```sh
docker buildx build --load -t ubuntu-qemu-arm .
```

The workspace level `.cargo/config.toml` file is set up to use the Docker container as runner for the Facade target.
The default configuration works for Linux and macOS, but must be changed for Windows hosts due to filepath incompatibilities.

**For Windows User:** Uncomment the runner configuration for Windows and comment the one for Linux and macOS.

### Ferrocene Setup

The Facade targets are special targets available with Ferrocene.
Similar to rustup, we provide [criticalup](https://criticalup.ferrocene.dev) to manage Ferrocene installations.

Assuming `criticalup` is installed, authenticate and install Ferrocene via:

```sh
# This will ask you for an authentication token
criticalup auth set
# This will install Ferrocene as configured in 'criticalup.toml'
criticalup install
```

This repository has set `ferrocene` as default rustup channel in `rust-toolchain.toml`
to ensure Ferrocene is used for all Cargo commands.

**To make Ferrocene available for rustup, run:**

```sh
criticalup link create
```

## Hardware Mapping

**RAD:**
- Outputs
  - p1.01: Radiation Control
    - Low: Start Radiation
    - High: Stop Radiation
  - p1.02: Mode Indicator
    - Low: Operation
    - High: Idle
- Inputs
  - p1.05: Start-Stop Switch
    - Low: Start
    - High: Stop
  - p1.06: Door Sensor
    - Low: Closed
    - High: Open
  - p1.07: Confirmation Switch
    - Low: Closed
    - High: Open
  - p1.08: Radiation Sensor (TODO: replace with analog input)
    - Low: Deactive
    - High: Active
- LEDS
  - 1: Mode Indicator (ON = Operation)
  - 2: Door Sensor (ON = Closed)
  - 3: Confirmation Switch (ON = Confirmed)
  - 4: Radiation Relay


**Sim:**
- Outputs
  - p1.01: Start-Stop Switch
    - Low: Start
    - High: Stop
  - p1.02: Door Sensor
    - Low: 
- Inputs
- LEDs
