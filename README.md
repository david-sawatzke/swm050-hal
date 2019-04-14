# swm050-hal

A hal for swm050 chips. Large portions
of this hal are based on the
[_stm32f0xx-hal_](https://github.com/stm32-rs/stm32f0xx-hal) hal.

## Flashing

I use the branch from
https://github.com/blacksphere/blackmagic/pull/401.
Other options are using the jlink with some plugin by synwit or this 
[openocd](http://openocd.zylin.com/#/c/4927/) branch. You could also try using
pyOCD with the pack file from the manufacturer.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
