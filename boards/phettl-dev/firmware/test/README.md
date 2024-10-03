# XIAO Seeed Studio nRF5280: Rust

This project provides a basic example on how to get started with Rust on a [Seeed Studio XIAO nRF52840 board](https://www.seeedstudio.com/Seeed-XIAO-BLE-nRF52840-p-5201.html?gclid=Cj0KCQjw6_CYBhDjARIsABnuSzrcbBi2wLRB5l0KG4IPh65GQ9RgmUYoG9iAbDoeJZrzz2nTkZ1sDyUaAonwEALw_wcB)

The `src/bin` folder provides some basic examples to get started, including blinking the onboard LED.

## UF2 bootloader

The Seeed Studio XIAO nRF52840 comes standard with UF2 bootloader similar to what many adafruit boards provide

We can make use of this feature to provide an easy way to flash the device that does not require external hardware, just a USB connection. 

For convenience, the UF2 python script is wrapped and provided as a cargo runner in the project.

The UF2 flashing procedure is very simple:

1. Connect the Seeed Studio XIAO nRF52840 with a USB cable

2. Double click the reset button quickly (~500ms) to get into bootloader mode

3. `cargo run [--bin blinly]` to flash the blinking LED example
    ```bash
    cargo run --bin blinky 
    Compiling nrf v0.1.0 (/Users/dasnaghi/Sandbox/Rust/nrf)
        Finished dev [unoptimized + debuginfo] target(s) in 1.39s
        Running `/Users/dasnaghi/Sandbox/Rust/nrf/tools/uf2 --base 0x27000 -f 0xADA52840 target/thumbv7em-none-eabihf/debug/blinky`
    Converted to uf2, output size: 108032, start address: 0x27000
    Flashing /Volumes/XIAO-SENSE (Seeed_XIAO_nRF52840_Sense)
    Wrote 108032 bytes to /Volumes/XIAO-SENSE/NEW.UF2
    ```

4. If the board is not in bootloader mode, this will be the output instead
    ```bash
    cargo run --bin blinky
        Finished dev [unoptimized + debuginfo] target(s) in 0.28s
        Running `/Users/dasnaghi/Sandbox/Rust/nrf/tools/uf2 --base 0x27000 -f 0xADA52840 target/thumbv7em-none-eabihf/debug/blinky`
    Converted to uf2, output size: 108032, start address: 0x27000
    No drive to deploy.
    ```

## Cargo Embed

In addition to the bootloader we can also flash the probe with the `cargo embed` command (after installing [`cargo embed`](https://github.com/probe-rs/cargo-embed)). This will also provide `rtt` logging and `gdb` debugging capabilities

This requires a probe (ST-LINK, JLINK, etc.) and a SWDIO/SWCLK connection, so it is usually reserved for more advanced use cases. 

The [XIAO expansion board](https://wiki.seeedstudio.com/Seeeduino-XIAO-Expansion-Board/) will provide easier access to these pins

## Advancde Notes:

### Linking 

In order to work with the softdevice and make sure everything is fine, we need to provide a custom linker that offsets the binary in FLASH. This can be seen in [`memory.x`](./memory.x)

```
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF52840 with Softdevices S140 7.3.0 */
  FLASH : ORIGIN = 0x00027000, LENGTH = 868K
  RAM : ORIGIN = 0x20020000, LENGTH = 128k
}
```

This custom linker (and some more linker things) are enabled by [`build.rs`](./build.rs). This is extremely important to know what offset our binary is at for the [UF2 bootloader settings](#uf2-bootloader-settings)

### UF2 bootloader settings

As previously mentioned, we need to know where our binary lives in memory to make sure we can place it there when creating the UF2 image. This can be done with the following arguments

`--base 0x27000 --family 0xADA52840`

`--base` is set to `0x27000` since that's where we told out linker the binary would be. If we changed it in the linker we would need to change it here too

`--family` is the flag that tags the UF2 image as safe to flash on our hardware, for the nRF5284 this value is `0xADA52840`
