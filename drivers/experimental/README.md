# `drivers`

A modular, cross-platform Rust crate for hardware drivers within the **OpenFab ecosystem**.

---

## Overview

The `drivers` crate is designed as part of **OpenFab**, a modular fabrication platform that makes it easy to integrate and control a wide variety of fabrication equipment. The goal of this crate is to provide:

- **Hardware-agnostic abstractions** over GPIO, SPI, UART, PWM, and other peripherals using [`embedded-hal`](https://docs.rs/embedded-hal/1.0.0/embedded_hal/).  
- **Cross-platform support** for MCUs like RP2040, STM32, and ESP32, as well as full systems (desktop computers) acting as driver nodes.  
- **Flexible runtime environments** with feature-gated WASM execution using [`wasmi`](https://docs.rs/wasmi/latest/wasmi/) on embedded systems or [`wasmtime`](https://docs.rs/wasmtime/latest/wasmtime/) on full systems.  
- **Integration into OpenFab’s IPP frontend**, enabling seamless delivery of print and fabrication jobs to 3D printers, BW printers, laser cutters, CNC routers, and more.

The crate is structured to allow **drivers to be slotted in or swapped** with minimal friction, making it easy to extend and maintain.

---

## Features

| Feature | Description |
|---------|-------------|
| `serialport` | Enable standard serial communication (desktop/driver computers) |
| `rpi` | Enable Raspberry Pi GPIO and peripherals via [`rppal`](https://docs.rs/rppal/latest/rppal/) |
| `esp32` | Enable ESP32 HAL for embedded control |
| `wasmi` | Enable embedded WebAssembly interpreter for MCU targets |
| `wasi` | Enable WASI support for full system / desktop targets |
| `wasmtime-wasi` | Enable Wasmtime WASI runtime for desktop/full system execution |

Feature flags allow you to **compile only the necessary HALs or runtimes** for your target platform, keeping builds small and portable.

---

## Getting Started

Add `drivers` to your `Cargo.toml`:

```toml
[dependencies]
drivers = { path = "../drivers", features = ["rpi", "wasmi"] }
```

### Use feature flags to target your platform:

- RP2040 / STM32 / ESP32 embedded targets: enable wasmi + the relevant HAL feature.

- Desktop / full system targets: enable wasmtime-wasi or wasi.

- Raspberry Pi: enable rpi.

- Serial communication: enable serialport.

## Design Philosophy

- Modular: Each MCU HAL or runtime is optional and feature-gated.

- Cross-platform: Abstracts the differences between MCUs and full systems to allow the same driver code to run everywhere.

- Composable: Drivers can plug directly into OpenFab’s IPP frontend for task dispatching.

- Future-proof: Easy to extend to new MCUs, HALs, or runtime environments.

## Supported Platforms (planned)
