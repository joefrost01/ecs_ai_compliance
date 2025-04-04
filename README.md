# AI Compliance Monitoring System

A high-performance ECS-based system for monitoring AI service usage compliance in real-time.

## Overview

This project demonstrates how Entity Component System (ECS) architecture, traditionally used in game development, can be applied to create blazingly fast AI compliance monitoring pipelines. The system processes AI service usage events, applies compliance rules (EU AI Act, GDPR, internal policies), and provides real-time metrics through a terminal-based dashboard.

## Features

- **Ultra-high performance**: Processes 70+ million AI service usage events per second
- **ECS architecture**: Efficient data organization and processing
- **Multi-threaded processing**: Utilizes all available CPU cores
- **Real-time TUI dashboard**: Visualizes compliance metrics as they're processed
- **Configurable rules**: Demonstrates different types of compliance checks
- **Low memory footprint**: Components are kept small for cache efficiency


## Getting Started

### Prerequisites

- Rust and Cargo (1.85.0 or newer)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/joefrost01/ecs_ai_compliance.git
cd ai-compliance-monitor
```

2. Build the project:
```bash
cargo build --release
```

### Running the Application

Run with default settings:
```bash
cargo run --release
```

Or specify custom parameters:
```bash
cargo run --release -- --rate 500000 --interval 2 --threads 8
```

### Command Line Arguments

- `--rate, -r`: Number of AI events to process per second (default: 100000)
- `--interval, -i`: Reporting interval in seconds (default: 5)
- `--threads, -t`: Number of worker threads (defaults to number of logical cores)

## Architecture

The system uses the Entity Component System (ECS) architecture:

- **Entities**: AI service usage events
- **Components**:
    - `AIService`: Service name and vendor
    - `Usage`: Department and data sensitivity information
    - `ComplianceStatus`: Bit flags for compliance states
    - `RiskAssessment`: Risk score and factor flags

- **Systems**:
    - EU AI Act compliance rules
    - GDPR compliance rules
    - Internal policy rules
    - Risk assessment

## Dashboard Navigation

The TUI dashboard provides four main views:

- **Overview**: General statistics and processing rates
- **Services**: Breakdown of AI service and vendor usage
- **Compliance**: Compliance status and violations
- **Risk**: Risk distribution and factors

Navigation:
- Press `1-4` to switch between tabs
- Press `Tab` to cycle through tabs
- Press `q` or `Esc` to exit

## Performance Notes

The system is designed to demonstrate the theoretical limits of compliance rule processing. In a real-world implementation, additional factors like database writes, API calls, and network latency would impact performance.

On a MacBook Pro M2-Max, the system processes an AI compliance event in approximately 14 nanoseconds - less time than it takes light to travel across a room!

## Project Structure

```
├── src/
│   ├── main.rs           - Application entry point
│   ├── components.rs     - ECS components and CLI args
│   ├── constants.rs      - Shared constants
│   ├── ecs.rs            - ECS systems and logic
│   ├── metrics.rs        - Metrics collection and processing
│   └── ui/
│       ├── mod.rs        - UI module definition
│       ├── dashboard.rs  - TUI dashboard implementation
│       ├── tui.rs        - Terminal setup/teardown
│       └── widgets.rs    - Reusable UI components
├── Cargo.toml
└── README.md
```

## Using in Your Own Projects

While this is primarily a demonstration, the approach can be adapted for real-world compliance monitoring:

1. Define your compliance components based on your specific requirements
2. Create systems for each compliance rule or policy
3. Integrate with your event sources (APIs, log files, etc.)
4. Add database persistence for compliance records
5. Extend the dashboard to include your specific metrics

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- The `hecs` ECS library
- The `tui-rs` and `crossterm` libraries for terminal UI
- The Rust game development community for ECS inspiration
