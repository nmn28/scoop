#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nrf52840_hal as hal;
use hal::{pac, twim, gpio};
use hal::prelude::*;
use panic_halt as _;

// MAX30102 I2C Address
const MAX30102_ADDR: u8 = 0x57;

// MAX30102 Registers
const REG_INTR_STATUS_1: u8 = 0x00;
const REG_INTR_STATUS_2: u8 = 0x01;
const REG_INTR_ENABLE_1: u8 = 0x02;
const REG_FIFO_WR_PTR: u8 = 0x04;
const REG_FIFO_RD_PTR: u8 = 0x06;
const REG_FIFO_DATA: u8 = 0x07;
const REG_FIFO_CONFIG: u8 = 0x08;
const REG_MODE_CONFIG: u8 = 0x09;
const REG_SPO2_CONFIG: u8 = 0x0A;
const REG_LED1_PA: u8 = 0x0C;
const REG_LED2_PA: u8 = 0x0D;
const REG_PILOT_PA: u8 = 0x0E;
const REG_PART_ID: u8 = 0xFF;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let port0 = gpio::p0::Parts::new(p.P0);
    
    // Configure I2C pins (P0.12 and P0.11 for XIAO nRF52840)
    let scl = port0.p0_12.into_floating_input().degrade();
    let sda = port0.p0_11.into_floating_input().degrade();

    // Initialize TWIM (I2C) peripheral
    let mut i2c = twim::Twim::new(
        p.TWIM0,
        twim::Pins { scl, sda },
        twim::Frequency::K100,
    );

    // Initialize UARTE (Serial) peripheral
    let uart_pins = hal::uarte::Pins {
        txd: port0.p0_20.into_push_pull_output(gpio::Level::High).degrade(),
        rxd: port0.p0_19.into_floating_input().degrade(),
        cts: None,
        rts: None,
    };

    let mut uart = hal::uarte::Uarte::new(
        p.UARTE0,
        uart_pins,
        hal::uarte::Parity::EXCLUDED,
        hal::uarte::Baudrate::BAUD115200,
    );

    // Send startup message with version info
    let _ = uart.write(b"MAX30102 Test v1.0\r\n");
    let _ = uart.write(b"Starting initialization...\r\n");

    // Initial delay to let sensor power up
    let _ = uart.write(b"Waiting for sensor power up (1 second)...\r\n");
    cortex_m::asm::delay(64_000_000); // 1 second delay at 64MHz
    let _ = uart.write(b"Power up complete\r\n");

    // Check if sensor is present
    let mut part_id = [0u8];
    match i2c.write_read(MAX30102_ADDR, &[REG_PART_ID], &mut part_id) {
        Ok(_) => {
            let mut id_buffer = [0u8; 32];
            let msg = format_u32(part_id[0] as u32, &mut id_buffer);
            let _ = uart.write(b"Found sensor with ID: ");
            let _ = uart.write(msg);
            let _ = uart.write(b"\r\n");
        },
        Err(_) => {
            let _ = uart.write(b"Error: Could not find MAX30102 sensor\r\n");
            loop {
                cortex_m::asm::delay(8_000_000);
            }
        }
    }

    // Read and verify part ID
    let mut part_id = [0u8];
    match i2c.write_read(MAX30102_ADDR, &[REG_PART_ID], &mut part_id) {
        Ok(_) => {
            let _ = uart.write(b"Found MAX30102 with ID: ");
            let mut id_buffer = [0u8; 32];
            let msg = format_u32(part_id[0] as u32, &mut id_buffer);
            let _ = uart.write(msg);
            let _ = uart.write(b"\r\n");
            
            if part_id[0] != 0x15 { // Expected part ID for MAX30102
                let _ = uart.write(b"Warning: Unexpected part ID\r\n");
            }
        },
        Err(_) => {
            let _ = uart.write(b"Error: Could not read part ID\r\n");
            loop {
                cortex_m::asm::delay(8_000_000);
            }
        }
    }

    // Initialize MAX30102
    match init_max30102(&mut i2c, &mut uart) {
        Ok(_) => {
            let _ = uart.write(b"MAX30102 initialized successfully\r\n");
            
            let mut msg_buffer = [0u8; 32];
            
            // Clear FIFO before starting
            let _ = uart.write(b"Clearing FIFO...\r\n");
            if let Err(_) = i2c.write(MAX30102_ADDR, &[REG_FIFO_WR_PTR, 0]) {
                let _ = uart.write(b"Failed to clear write pointer\r\n");
                loop {
                    cortex_m::asm::delay(8_000_000);
                }
            }
            if let Err(_) = i2c.write(MAX30102_ADDR, &[REG_FIFO_RD_PTR, 0]) {
                let _ = uart.write(b"Failed to clear read pointer\r\n");
                loop {
                    cortex_m::asm::delay(8_000_000);
                }
            }
            
            // Verify FIFO pointers
            let mut wr_ptr = [0u8];
            let mut rd_ptr = [0u8];
            if let Err(_) = i2c.write_read(MAX30102_ADDR, &[REG_FIFO_WR_PTR], &mut wr_ptr) {
                let _ = uart.write(b"Failed to read write pointer\r\n");
                loop {
                    cortex_m::asm::delay(8_000_000);
                }
            }
            if let Err(_) = i2c.write_read(MAX30102_ADDR, &[REG_FIFO_RD_PTR], &mut rd_ptr) {
                let _ = uart.write(b"Failed to read read pointer\r\n");
                loop {
                    cortex_m::asm::delay(8_000_000);
                }
            }
            
            let _ = uart.write(b"FIFO pointers - Write: ");
            let msg = format_u32(wr_ptr[0] as u32, &mut msg_buffer);
            let _ = uart.write(msg);
            let _ = uart.write(b", Read: ");
            let msg = format_u32(rd_ptr[0] as u32, &mut msg_buffer);
            let _ = uart.write(msg);
            let _ = uart.write(b"\r\n");
            
            let _ = uart.write(b"Place your finger on the sensor and hold still...\r\n");
            let _ = uart.write(b"It may take a few seconds to get stable readings\r\n");
        },
        Err(_) => {
            let _ = uart.write(b"Failed to initialize MAX30102\r\n");
            loop {
                cortex_m::asm::delay(8_000_000);
            }
        }
    }

    let mut buffer = [0u8; 6];
    let mut msg_buffer = [0u8; 32];
    let mut samples_buffer = [0u8; 32];
    let mut last_status_print = 0u32;

    let _ = uart.write(b"Entering main loop...\r\n");
    loop {
        // Check interrupt status
        let mut intr_status = [0u8];
        match i2c.write_read(MAX30102_ADDR, &[REG_INTR_STATUS_1], &mut intr_status) {
            Ok(_) => {
                // Print status periodically (every ~1 second)
                if last_status_print >= 400 { // Adjusted for 400Hz sample rate
                    let _ = uart.write(b"[DEBUG] Heartbeat - System running\r\n");
                    let _ = uart.write(b"[DEBUG] Interrupt Status: 0x");
                    let msg = format_u32(intr_status[0] as u32, &mut msg_buffer);
                    let _ = uart.write(msg);
                    let _ = uart.write(b"\r\n");

                    // Read and display current LED settings
                    let mut led_settings = [0u8; 2];
                    match i2c.write_read(MAX30102_ADDR, &[REG_LED1_PA], &mut led_settings) {
                        Ok(_) => {
                            let _ = uart.write(b"[DEBUG] LED Settings - Red: 0x");
                            let msg = format_u32(led_settings[0] as u32, &mut msg_buffer);
                            let _ = uart.write(msg);
                            let _ = uart.write(b"\r\n");
                        },
                        Err(_) => {
                            let _ = uart.write(b"[ERROR] Failed to read LED settings\r\n");
                        }
                    }

                    last_status_print = 0;
                } else {
                    last_status_print += 1;
                }

                // Check if FIFO is ready
                if intr_status[0] & 0x80 != 0 { // FIFO Almost Full Flag
                    let _ = uart.write(b"[INFO] FIFO interrupt triggered\r\n");
                    // Read FIFO
                    match read_fifo(&mut i2c) {
                        Ok(samples) => {
                            let _ = uart.write(b"[DEBUG] FIFO samples available: ");
                            let msg = format_u32(samples as u32, &mut samples_buffer);
                            let _ = uart.write(msg);
                            let _ = uart.write(b"\r\n");

                            if samples > 0 {
                                // Read one sample (LED1 = Red, LED2 = IR)
                                match i2c.write_read(MAX30102_ADDR, &[REG_FIFO_DATA], &mut buffer) {
                                    Ok(_) => {
                                        let _ = uart.write(b"[DEBUG] Raw FIFO data: [");
                                        for byte in &buffer {
                                            let msg = format_u32(*byte as u32, &mut msg_buffer);
                                            let _ = uart.write(msg);
                                            let _ = uart.write(b" ");
                                        }
                                        let _ = uart.write(b"]\r\n");

                                        // Process the sample (Red LED data)
                                        let red = ((buffer[0] as u32) << 16 |
                                                (buffer[1] as u32) << 8 |
                                                (buffer[2] as u32)) & 0x3FFFF;
                                        
                                        let ir = ((buffer[3] as u32) << 16 |
                                                (buffer[4] as u32) << 8 |
                                                (buffer[5] as u32)) & 0x3FFFF;

                                        // Send detailed sensor values with signal quality indicators
                                        let _ = uart.write(b"[INFO] Signal Quality: ");
                                        if red > 10000 && ir > 10000 {
                                            let _ = uart.write(b"Good - Sensor detecting tissue\r\n");
                                        } else {
                                            let _ = uart.write(b"Poor - Adjust finger position\r\n");
                                        }
                                        
                                        let _ = uart.write(b"RED=");
                                        let msg = format_u32(red, &mut msg_buffer);
                                        let _ = uart.write(msg);
                                        let _ = uart.write(b" IR=");
                                        let msg = format_u32(ir, &mut msg_buffer);
                                        let _ = uart.write(msg);
                                        let _ = uart.write(b"\r\n");

                                        // Print signal-to-noise ratio estimate
                                        let ratio = if ir > 0 { (red as f32) / (ir as f32) } else { 0.0 };
                                        if ratio > 0.5 && ratio < 2.0 {
                                            let _ = uart.write(b"[INFO] R/IR Ratio OK - Continue measuring\r\n");
                                        } else {
                                            let _ = uart.write(b"[WARN] R/IR Ratio outside normal range - Reposition finger\r\n");
                                        }
                                    },
                                    Err(_) => {
                                        let _ = uart.write(b"[ERROR] Failed to read FIFO data\r\n");
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            let _ = uart.write(b"[ERROR] Failed to read FIFO pointers\r\n");
                        }
                    }
                }
            },
            Err(_) => {
                let _ = uart.write(b"[ERROR] Failed to read interrupt status\r\n");
            }
        }

        // Delay ~2.5ms at 64MHz (400Hz sample rate)
        cortex_m::asm::delay(160_000);
    }
}

fn init_max30102(i2c: &mut twim::Twim<hal::pac::TWIM0>, uart: &mut hal::uarte::Uarte<hal::pac::UARTE0>) -> Result<(), ()> {
    // Reset the sensor
    let _ = uart.write(b"Resetting sensor...\r\n");
    i2c.write(MAX30102_ADDR, &[REG_MODE_CONFIG, 0x40]).map_err(|_| ())?;
    cortex_m::asm::delay(32_000_000); // Delay ~500ms at 64MHz for full reset

    // Configure sensor for heart rate mode
    let config = [
        // FIFO Configuration
        (REG_FIFO_CONFIG, 0x1F),  // Sample averaging = 4, FIFO rollover = true
        // Mode Configuration
        (REG_MODE_CONFIG, 0x02),  // Heart rate mode with red LED only
        // SPO2 Configuration
        (REG_SPO2_CONFIG, 0x27),  // SPO2 ADC range = 4096nA, Sample rate = 400Hz
        // LED Pulse Amplitude - Start with lower power to avoid saturation
        (REG_LED1_PA, 0x3F),     // Red LED = ~12.5mA
        (REG_LED2_PA, 0x00),     // IR LED off for heart rate mode
        // Enable FIFO_RDY and FIFO_A_FULL interrupts
        (REG_INTR_ENABLE_1, 0x80),
    ];

    let mut reg_buffer = [0u8; 32];
    let mut val_buffer = [0u8; 32];

    let _ = uart.write(b"Configuring sensor with settings:\r\n");
    for &(reg, val) in config.iter() {
        let _ = uart.write(b"Register 0x");
        let msg = format_u32(reg as u32, &mut reg_buffer);
        let _ = uart.write(msg);
        let _ = uart.write(b" = 0x");
        let msg = format_u32(val as u32, &mut val_buffer);
        let _ = uart.write(msg);
        let _ = uart.write(b"\r\n");
        
        i2c.write(MAX30102_ADDR, &[reg, val]).map_err(|_| ())?;
        
        // Verify the write
        let mut readback = [0u8];
        if let Ok(_) = i2c.write_read(MAX30102_ADDR, &[reg], &mut readback) {
            if readback[0] != val {
                let _ = uart.write(b"Warning: Register write verification failed\r\n");
            }
        }
    }

    Ok(())
}

fn read_fifo(i2c: &mut twim::Twim<hal::pac::TWIM0>) -> Result<u8, ()> {
    let mut write_ptr = [0u8];
    let mut read_ptr = [0u8];

    // Read FIFO Write Pointer
    i2c.write_read(MAX30102_ADDR, &[REG_FIFO_WR_PTR], &mut write_ptr).map_err(|_| ())?;
    
    // Read FIFO Read Pointer
    i2c.write_read(MAX30102_ADDR, &[REG_FIFO_RD_PTR], &mut read_ptr).map_err(|_| ())?;

    // Calculate number of samples available
    Ok((write_ptr[0].wrapping_sub(read_ptr[0])) & 0x1F)
}

fn format_u32(num: u32, buffer: &mut [u8]) -> &[u8] {
    let mut idx = 0;
    let mut n = num;
    let mut digits = [0u8; 10];
    let mut digit_count = 0;

    // Convert to digits
    if n == 0 {
        digits[0] = b'0';
        digit_count = 1;
    } else {
        while n > 0 {
            digits[digit_count] = (n % 10) as u8 + b'0';
            n /= 10;
            digit_count += 1;
        }
    }

    // Reverse digits into buffer
    for i in (0..digit_count).rev() {
        buffer[idx] = digits[i];
        idx += 1;
    }

    &buffer[..idx]
}