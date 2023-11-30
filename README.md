# Platform-agnostic driver for TM1637

This repository contains a driver for seven-segment display TM1637.
It was implemented using `embedded-hal`, so it should be platform-agnostic,
though examples use ESP32. Several examples are included, e. g. simple digital
[clocks](https://github.com/SeTSeR/esp32-tm1637/blob/master/examples/clocks.rs)
or just [sending some bytes around](https://github.com/SeTSeR/esp32-tm1637/blob/master/examples/send-digits.rs).

# Checking out

Building:
```sh
cargo build
```

Running an example:
```sh
cargo run --profile=lto --example=clocks
```
