# XIAO Seeed Studio nRF5280: Rust

This project provides examples for working with the [Seeed Studio XIAO nRF52840 board](https://www.seeedstudio.com/Seeed-XIAO-BLE-nRF52840-p-5201.html), including heart rate monitoring using the MAX30102 sensor.

## Examples

The `src/bin` folder provides several examples:
- `blinky.rs`: Basic LED blinking example
- `console.rs`: Serial console communication
- `max30102.rs`: Heart rate monitoring using the MAX30102 sensor

### MAX30102 Heart Rate Monitor

The MAX30102 example demonstrates how to:
- Initialize and configure the MAX30102 sensor
- Read heart rate data at 400Hz
- Process and validate sensor readings
- Output data via serial for visualization

Features:
- Heart rate mode using red LED
- Automatic signal quality detection
- Detailed debug output
- Real-time data visualization

To use the heart rate monitor:
1. Build and flash the firmware
2. Run the Python visualization tool
3. Place your finger on the sensor
4. View real-time readings in the web interface

## UF2 bootloader

The Seeed Studio XIAO nRF52840 comes standard with UF2 bootloader similar to what many adafruit boards provide.

We can make use of this feature to provide an easy way to flash the device that does not require external hardware, just a USB connection. 

For convenience, the UF2 python script is wrapped and provided as a cargo runner in the project.

The UF2 flashing procedure is very simple:

1. Connect the Seeed Studio XIAO nRF52840 with a USB cable

2. Double click the reset button quickly (~500ms) to get into bootloader mode

3. Build and flash an example:
    ```bash
    # For the heart rate monitor:
    cargo build --release --bin max30102
    python3 tools/uf2conv.py -f 0xada52840 -c target/thumbv7em-none-eabihf/release/max30102 -o flash.uf2
    # Copy flash.uf2 to the mounted XIAO drive
    
    # For the blinky example:
    cargo run --bin blinky 
    ```

4. If the board is not in bootloader mode, this will be the output instead:
    ```bash
    cargo run --bin blinky
        Finished dev [unoptimized + debuginfo] target(s) in 0.28s
        Running `/Users/dasnaghi/Sandbox/Rust/nrf/tools/uf2 --base 0x27000 -f 0xADA52840 target/thumbv7em-none-eabihf/debug/blinky`
    Converted to uf2, output size: 108032, start address: 0x27000
    No drive to deploy.
    ```

## Python Visualization Tool

The project includes a Python script (`scoop_sensor_display.py`) that provides:
- Real-time data visualization via web interface
- Signal quality indicators
- Raw data display
- Interactive charts

To use the visualization tool:
1. Flash the MAX30102 firmware
2. Run `python3 scoop_sensor_display.py`
3. Open `http://localhost:8000` in your browser
4. Follow the on-screen instructions

## Cargo Embed

In addition to the bootloader we can also flash the probe with the `cargo embed` command (after installing [`cargo embed`](https://github.com/probe-rs/cargo-embed)). This will also provide `rtt` logging and `gdb` debugging capabilities.

This requires a probe (ST-LINK, JLINK, etc.) and a SWDIO/SWCLK connection, so it is usually reserved for more advanced use cases. 

The [XIAO expansion board](https://wiki.seeedstudio.com/Seeeduino-XIAO-Expansion-Board/) will provide easier access to these pins.

## Advanced Notes:

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

This custom linker (and some more linker things) are enabled by [`build.rs`](./build.rs). This is extremely important to know what offset our binary is at for the [UF2 bootloader settings](#uf2-bootloader-settings).

### UF2 bootloader settings

As previously mentioned, we need to know where our binary lives in memory to make sure we can place it there when creating the UF2 image. This can be done with the following arguments:

`--base 0x27000 --family 0xADA52840`

`--base` is set to `0x27000` since that's where we told out linker the binary would be. If we changed it in the linker we would need to change it here too.

`--family` is the flag that tags the UF2 image as safe to flash on our hardware, for the nRF5284 this value is `0xADA52840`.
