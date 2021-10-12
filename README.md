# qemu-fw-cfg-rs

A Rust library for reading fw_cfg from QEMU.

## Usage

Add the following to your `Cargo.toml`:

```yaml
[dependencies]
qemu-fw-cfg = "0.1"
```

To use `qemu-fw-cfg` without `alloc`, you can use this instead:

```yaml
[dependencies]
qemu-fw-cfg = { version = "0.1", default-features = false }
```

## Examples

```rust
use qemu_fw_cfg::FwCfg;

// Verify that we are inside QEMU.
if running_in_qemu() {
    // Create a new `FwCfg` instance.
    let fw_cfg = unsafe { FwCfg::new() };
    // Retrieve information of a file.
    let file = fw_cfg.find_file("etc/igd-opregion").unwrap();
    // Read data from the file.
    let data = fw_cfg.read_file(&file);
}
```

## Rust support

Currently, `qemu-fw-cfg` required nightly compiler to build.

## License

This project is licensed under either of Apache License, Version 2.0 or MIT license at your option. 
