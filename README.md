# Rusty Log Analyzer

A high-performance, concurrent log file analyzer written in Rust.

This command-line tool is designed to process massive log files (gigabytes in size) far faster than traditional scripting languages could. It leverages Rust's fearless concurrency and performance to extract meaningful data from logs quickly and efficiently.

## Core Features

- **Concurrent File Processing:** Reads and processes multiple log files simultaneously using `rayon`.
- **High-Performance Parsing:** Uses the `regex` crate for optimized log entry parsing.
- **Data Aggregation:** Counts IP addresses, HTTP status codes, and methods.
- **Efficient Memory Management:** Streams files line-by-line with batched updates.
- **Advanced Filtering:** Filter logs by HTTP status code or method.
- **Search Capability:** Search for specific IP addresses or patterns.
- **Export Options:** Export results to JSON, CSV, or text format.
- **Thread Control:** Specify number of threads for processing.
- **Verbose Mode:** Get detailed processing information.
- **Field Extraction:** Extract and analyze HTTP methods, status codes, paths, and more.

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

```bash
rusty [OPTIONS] <FILES>...
```

#### Options

- `-v, --verbose`: Enable verbose output with detailed processing information
- `-s, --search <PATTERN>`: Search for specific IP addresses or patterns
- `-t, --threads <NUM>`: Specify number of threads (default: all available)
- `-e, --export <FILE>`: Export results to file (JSON/CSV/text based on extension)
- `-a, --all`: Show all results instead of just top 20
- `--extract-fields`: Extract and analyze HTTP methods, status codes, etc.
- `--status <CODE>`: Filter logs by HTTP status code
- `--method <METHOD>`: Filter logs by HTTP method (GET, POST, etc.)

#### Examples

**Basic usage:**
```bash
./target/release/rusty access.log error.log
```

**Extract detailed fields with verbose output:**
```bash
./target/release/rusty logs/*.log --extract-fields --verbose
```

**Search for specific IP pattern:**
```bash
./target/release/rusty access.log --search "192.168"
```

**Filter by status code and export to JSON:**
```bash
./target/release/rusty access.log --extract-fields --status 404 --export errors.json
```

**Analyze with limited threads:**
```bash
./target/release/rusty large.log --threads 4
```

**Show all IPs (not just top 20):**
```bash
./target/release/rusty access.log --all
```

#### Sample Output

Given a log file, the tool provides comprehensive analysis:

```
Analyzing 1 files...

Analysis complete in 2.41ms
Total lines processed: 10

--- IP Address Counts ---
192.168.1.1: 4
10.0.0.1: 2
127.0.0.1: 1
10.0.0.2: 1

--- HTTP Status Codes ---
200: 7
404: 1

--- HTTP Methods ---
GET: 7
POST: 1
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
