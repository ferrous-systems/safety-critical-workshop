# safety-critical-workshop

This repository contains the accompanying demo for the "Safety-Critical Rust Development with Ferrocene" workshop.

## Prerequisites

- `cargo install mantra --locked`
- `cargo install nextest --locked`
- `cargo install embsinth --locked`
- Docker installed and usable

## Building the docker container

- `docker buildx create --name multiarch-builder --use`
- `docker buildx inspect --bootstrap`
- For building locally: `docker buildx build --load -t ubuntu-qemu-arm .`

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
  - p1.08: Radiation Sensor
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
