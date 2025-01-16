# Scoop - Heart Rate Monitor Project

[![GitHub Repository](https://img.shields.io/badge/GitHub-nmn28/scoop-blue?logo=github)](https://github.com/nmn28/scoop)

A heart rate monitoring project using the Seeed Studio XIAO nRF52840 board and MAX30102 sensor. This project provides firmware and visualization tools for real-time heart rate monitoring.

## Project Structure

```
scoop/
├── boards/
│   └── scoop-dev/          # Hardware design files
│       ├── diodelib/       # Component libraries
│       ├── elec/          # PCB design files
│       └── firmware/      # Embedded firmware
│           └── test/      # Example code and tests
├── scoop_sensor_display.py # Python visualization tool
└── README.md
```

## Features

- Real-time heart rate monitoring
- Signal quality detection
- Web-based visualization interface
- Embedded Rust firmware
- Python data processing

## Quick Start

1. Navigate to the firmware directory:
   ```bash
   cd boards/scoop-dev/firmware/test
   ```

2. Follow the [firmware README](boards/scoop-dev/firmware/test/README.md) for detailed setup and usage instructions.

## Documentation

- [Firmware Documentation](boards/scoop-dev/firmware/test/README.md)
- [Hardware Design Files](boards/scoop-dev/)

## License

This project is licensed under the MIT License - see the LICENSE file for details.