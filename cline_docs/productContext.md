# Product Context

## Purpose
The Scoop project is a hardware device that measures heart rate using a MAX30102 sensor. It consists of a custom hardware board (scoop-dev) with firmware that interfaces with the sensor and transmits data to a computer.

## Problems Solved
- Provides real-time heart rate monitoring
- Makes sensor data accessible through a web interface
- Enables visualization of heart rate data

## How It Works
1. Hardware Layer:
   - Uses MAX30102 sensor for heart rate detection
   - Custom PCB (scoop-dev board) with necessary components
   - XIAO microcontroller for sensor interfacing

2. Software Stack:
   - Rust firmware running on the microcontroller
   - Python backend (scoop_sensor_display.py) for data processing
   - Web frontend for visualization
   - WebSocket communication between layers

3. Data Flow:
   - Sensor captures heart rate data
   - Firmware processes raw sensor data
   - Data sent via serial to Python backend
   - Backend forwards data to web frontend via WebSocket
   - Frontend displays real-time heart rate information