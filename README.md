# libwdi-rs

Rust bindings to [pbatard/libwdi](https://github.com/pbatard/libwdi): A Windows Driver Installation library for USB devices.

Raw bindings are available in the `libwdi-sys` package, while `libwdi` provides idiomatic Rust wrappers.

Check [libwdi wiki](https://github.com/pbatard/libwdi/wiki/Usage) for detailed documentation.
The "Basic usage" example using `libwdi` would be:
```rust
if let Ok(list) = wdi::CreateListOptions::new().create_list() {
    for dev in devices.iter() {
        println!("Installing driver for USB device: \"{}\" ({:04x}:{:04x})",
            dev.desc(), dev.vid(), dev.pid());
        libwdi::PrepareDriverOptions::new()
            .prepare_driver(dev, DEFAULT_DIR, INF_NAME)
            .and_then(|driver| driver.install_driver())
            .ok();
    }
}
```

## Limitations

* Currently a forked libwdi version is used which disables libusb0 and libusbK in `msvc/config.h`. Find a way to configure this via `libwdi-sys/build.rs`.
* Currently libwdi is linked statically. Provide crate features to configure linking mode.
