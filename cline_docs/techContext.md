# Technical Context

## Technologies Used

### Hardware
- MAX30102 Heart Rate Sensor
  - I2C interface
  - Programmable LED current
  - Configurable sample rate
- XIAO Development Board
  - nRF52840 microcontroller
  - USB connectivity
  - I2C support

### Firmware
- Rust Programming Language
  - embedded-hal traits
  - cortex-m runtime
  - nrf52840-hal
- Build Tools
  - cargo
  - uf2conv.py for flashing

### Backend
- Python
  - pyserial for serial communication
  - websockets for real-time data
  - http.server for frontend serving

### Frontend
- HTML/JavaScript
  - WebSocket client
  - Real-time visualization
  - Browser-based interface

## Development Setup

### Hardware Setup
1. Connect MAX30102 sensor to XIAO board via I2C
2. Connect XIAO board to computer via USB
3. Verify serial port connection

### Firmware Development
1. Install Rust and cargo
2. Build firmware:
   ```
   cd boards/scoop-dev/firmware/test
   cargo build --release --bin max30102
   ```
3. Flash firmware:
   ```
   python3 tools/uf2conv.py target/thumbv7em-none-eabihf/release/max30102 -c -f 0xADA52840
   ```

### Backend Development
1. Run Python script:
   ```
   python3 scoop_sensor_display.py
   ```
2. Access web interface at http://localhost:8000

## Technical Constraints
1. Sensor Limitations
   - LED current range: 0-51mA
   - Sample rate: 50-1000Hz
   - I2C clock speed constraints

2. Communication
   - Serial baud rate limitations
   - WebSocket latency considerations
   - Browser refresh rate limits

3. Processing
   - Real-time data processing requirements
   - Memory constraints on microcontroller
   - JavaScript performance limitations