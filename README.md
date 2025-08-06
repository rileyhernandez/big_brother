# Big Brother - Scale Monitoring System

A Rust application that monitors multiple physical scales using Phidget hardware and logs weight data to a SQLite database.

## Overview

Big Brother is a real-time scale monitoring system designed to track weight changes across multiple food ingredient containers. It continuously monitors connected scales, detects weight change events, and maintains a comprehensive log of all scale activity in a local SQLite database.

## Features

- **Multi-Scale Monitoring**: Simultaneously monitor multiple scales with different ingredients
- **Phidget Hardware Integration**: Connect to physical scales using Phidget load cell interfaces
- **Real-time Event Detection**: Automatically detect and classify weight change events
- **Robust Error Handling**: Graceful handling of hardware failures with automatic reconnection
- **Comprehensive Logging**: All scale events logged with timestamps, locations, and ingredient details
- **Heartbeat Monitoring**: Regular status checks to ensure scales remain connected
- **Configurable Settings**: Flexible configuration for multiple scale setups

## Hardware Requirements

- Phidget devices (supported models: LibraV0)
- Load cell sensors connected to Phidget interfaces
- Linux system for deployment

## Installation

1. **Clone the repository**:
   ```bash
   git clone git@github.com:rileyhernandez/big_brother.git
   cd big_brother
   ```

2. **Build the application**:
   ```bash
   cargo build --release
   ```

3. **Create configuration directory**:
   ```bash
   mkdir -p ~/.config/libra
   ```

## Configuration

Create a configuration file at `~/.config/libra/config.toml` with your scale settings:

```toml
[libra-0.config]
phidget-id = 716304
load-cell-id = 0
gain = 10095659.7359
offset = -27.5
location = "Kitchen"
ingredient = "Flour"

[libra-0.device]
model = "LibraV0"
number = 0

[libra-1.config]
phidget-id = 716800
load-cell-id = 0
gain = 4850591.56128
offset = 93.15464
location = "Pantry"
ingredient = "Sugar"

[libra-1.device]
model = "LibraV0"
number = 1
```

### Configuration Parameters

- `phidget-id`: Unique identifier for the Phidget device
- `load-cell-id`: ID of the load cell connected to the Phidget
- `gain`: Calibration gain value for accurate weight readings
- `offset`: Calibration offset value
- `location`: Physical location description
- `ingredient`: Name of the ingredient being monitored
- `model`: Scale model type
- `number`: Unique scale number for identification

## Usage

### Running the Application

```bash
# Production mode (info level logging)
./target/release/big-brother prod

# Debug mode (detailed logging)
./target/release/big-brother debug
```

### Command Line Arguments

- `debug`: Enable debug-level logging for troubleshooting
- `prod`: Production mode with info-level logging (default)

## Database Schema

The application creates a SQLite database at `~/.config/libra/data.db` with the following schema:

```sql
CREATE TABLE libra_logs (
    model TEXT NOT NULL,           -- Scale model (e.g., "LibraV0")
    number TEXT NOT NULL,          -- Scale number identifier
    timestamp TEXT NOT NULL,       -- ISO8601 timestamp
    action TEXT NOT NULL,          -- Action type (Starting, Heartbeat, Offline)
    amount NUMBER NOT NULL,        -- Weight amount in configured units
    location TEXT NOT NULL,        -- Scale location
    ingredient TEXT NOT NULL,      -- Ingredient being monitored
    synced INTEGER NOT NULL DEFAULT 0  -- Sync status flag (0 = not synced, 1 = synced)
);
```

## Scale Actions

The system tracks several types of scale events:

- **Starting**: Initial weight reading when scale connects or restarts
- **Heartbeat**: Periodic status update to confirm scale is operational
- **Offline**: Scale has disconnected or failed to respond

## System Architecture

### Core Components

- **main.rs**: Application entry point and main monitoring loop
- **data.rs**: Database operations and data structure definitions
- **error.rs**: Comprehensive error handling for all system components

### External Dependencies

- **Scale Library**: Custom Rust library for Phidget hardware communication
- **Menu Library**: Device identification and management system
- **SQLite**: Local data storage via rusqlite
- **Syslog**: System logging integration

### Monitoring Loop

The application runs a continuous monitoring loop that:

1. Reads current weight from all connected scales
2. Checks for weight change events and classifies actions
3. Handles hardware connection failures with automatic restart
4. Logs all events to the SQLite database
5. Maintains heartbeat status for all scales
6. Sleeps according to configured sample periods

## Logging

The application uses syslog for system-level logging:

- **Info Level**: Connection status, scale restarts, general operation
- **Warning Level**: Connection failures, hardware errors
- **Error Level**: Unrecoverable errors
- **Debug Level**: Detailed weight readings and system state

## Error Handling

Robust error handling includes:

- **Hardware Failures**: Automatic scale restart on Phidget errors
- **Connection Issues**: Graceful handling of disconnected devices
- **Database Errors**: Proper error propagation and logging
- **Configuration Errors**: Clear error messages for setup issues

## Development

### Building from Source

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Debug Mode

For development and troubleshooting, run with debug logging:

```bash
cargo run -- debug
```

## Dependencies

### Runtime Dependencies

- **thiserror**: Error handling macros
- **log**: Logging framework
- **rusqlite**: SQLite database interface
- **time**: Timestamp generation and formatting
- **syslog**: System logging integration
- **Custom scale library**: Hardware communication
- **Custom menu library**: Device management

### Build Dependencies

- Rust 2024 edition
- Cargo package manager

## File Structure

```
big_brother/
├── Cargo.toml              # Project dependencies and metadata
├── config.toml             # Example configuration file
├── src/
│   ├── main.rs            # Main application logic
│   ├── data.rs            # Database operations
│   └── error.rs           # Error type definitions
└── README.md              # This file
```

## Configuration Directory

```
~/.config/libra/
├── config.toml            # Scale configuration
└── data.db               # SQLite database
```
