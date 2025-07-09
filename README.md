# Rusty Log Analyzer

A high-performance, concurrent log file analyzer written in Rust.

This command-line tool is designed to process massive log files (gigabytes in size) far faster than traditional scripting languages could. It leverages Rust's fearless concurrency and performance to extract meaningful data from logs quickly and efficiently.

## Core Features

- **Concurrent File Processing:** Reads and processes multiple log files simultaneously using `rayon`.
- **High-Performance Parsing:** Uses the `regex` crate for optimized log entry parsing.
- **Data Aggregation:** Currently counts the occurrences of each IP address found in the logs.
- **Efficient Memory Management:** Streams files line-by-line, keeping memory usage low and predictable.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Building

Clone the repository and build the project using Cargo:

```bash
cargo build --release
```

The optimized binary will be located at `target/release/rusty`.

### Usage

To analyze one or more log files, pass their paths as command-line arguments:

```bash
./target/release/rusty /path/to/your/log1.log /path/to/your/log2.log
```

#### Example

Given a log file named `dummy.log`:

```log
127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326
192.168.1.1 - - [10/Oct/2000:13:55:36 -0700] "GET /asf_logo.gif HTTP/1.0" 200 2326
127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "POST /some/path HTTP/1.0" 404 2326
10.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /another/path HTTP/1.0" 200 2326
```

Running the tool will produce the following output:

```
Analyzing 1 files...
Analysis complete.
--- IP Address Counts ---
192.168.1.1: 1
10.0.0.1: 1
127.0.0.1: 2
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
