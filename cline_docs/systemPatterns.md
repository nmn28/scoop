# System Patterns

## Architecture Overview
The system follows a layered architecture:

1. Hardware Layer
   - MAX30102 sensor (I2C interface)
   - XIAO microcontroller board
   - Custom PCB for integration

2. Firmware Layer (Rust)
   - Embedded Rust for microcontroller
   - HAL abstractions for hardware access
   - Serial communication for data transfer

3. Backend Layer (Python)
   - Serial data collection
   - WebSocket server for real-time data
   - HTTP server for frontend

4. Frontend Layer (HTML/JavaScript)
   - WebSocket client for real-time updates
   - Data visualization
   - User interface

## Key Technical Decisions

### Firmware
- Using Rust for type safety and memory safety
- Embedded HAL traits for hardware abstraction
- Async/await for concurrent operations
- Serial communication for data transfer

### Communication
- Serial port for firmware-to-PC communication
- WebSocket for real-time data streaming
- HTTP for serving frontend assets

### Data Flow
1. Sensor → Firmware
   - I2C communication
   - Raw data processing
   - Serial transmission

2. Firmware → Backend
   - Serial port communication
   - Data parsing
   - WebSocket broadcasting

3. Backend → Frontend
   - WebSocket real-time updates
   - JSON data format
   - Error handling

## Design Patterns
1. Publisher/Subscriber
   - WebSocket for real-time updates
   - Event-driven architecture

2. State Machine
   - Sensor initialization
   - Data acquisition
   - Error handling

3. Factory Pattern
   - Hardware abstraction
   - Device initialization

4. Observer Pattern
   - Real-time data updates
   - UI updates