# tps546

Async, `no_std` driver for the Texas Instruments
[TPS546D24A](https://www.ti.com/product/TPS546D24A) stackable PMBus buck
converter, built on top of
[`pmbus-adapter`](https://crates.io/crates/pmbus-adapter) and
[`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).

## Features

- **Typed registers** — configuration and status registers are parsed into
  Rust structs and enums, not raw bytes.
- **Automatic voltage scaling** — `ULinear16` / `Linear11` encoding and
  decoding is handled transparently; read and write voltages, currents, and
  temperatures as `f32`.
- **Bulk telemetry** — `read_all()` returns Vout, Iout, temperature, and Vin
  in a single I2C transaction.
- **Manufacturer-specific commands** — telemetry config, stack/sync config,
  NVM checksum, fault simulation, and more.
- **`no_std` compatible** — zero heap allocations, suitable for bare-metal and
  RTOS targets.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
tps546 = "0.1"
```

```rust,no_run
use tps546::{Tps546, DEFAULT_ADDR, Operation};
use pmbus_adapter::PmbusAdaptor;
use smbus_adapter::SmbusAdaptor;

async fn example<B: embedded_hal_async::i2c::I2c>(bus: B) {
    let smbus = SmbusAdaptor::new(bus);
    let pmbus = PmbusAdaptor::new(smbus);
    let mut dev = Tps546::new(pmbus, DEFAULT_ADDR);

    // Query VOUT_MODE and cache the exponent
    dev.init().await.unwrap();

    // Set output voltage to 1.0 V
    dev.set_vout_command(1.0).await.unwrap();

    // Turn on the output (bit 7 = ON)
    dev.set_operation(Operation::from_raw(0x80)).await.unwrap();

    // Read telemetry in one shot
    let t = dev.read_all().await.unwrap();
    let _vout = t.vout_f32(dev.vout_exponent());
    let _iout = t.iout_f32();
    let _temp = t.temperature_f32();
}
```

## Minimum Supported Rust Version

This crate requires **Rust 1.85.1** or later (edition 2024).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
