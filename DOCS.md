# Project Documentation

This file documents the steps taken to build the Rust project.

## Step 1: Initialize Project

- Initialized a new Rust project using `cargo init`. This created a `Cargo.toml` file for managing dependencies and a `src` directory containing a basic `main.rs` file.

## Step 2: Project Scaffolding and Dependencies

- **Goal:** Build a high-performance, concurrent log file analyzer.
- **Core Features:**
    - **Concurrent File Processing:** Use `rayon` to process multiple files or file chunks in parallel.
    - **High-Performance Parsing:** Use `regex` for efficient log entry parsing.
    - **Data Aggregation:** Count occurrences of specific patterns (e.g., errors, IP addresses).
    - **Efficient Memory Management:** Process files line-by-line to keep memory usage low.
- **Dependencies added:**
    - `clap`: For parsing command-line arguments.
    - `rayon`: For parallel processing.
    - `regex`: For parsing log lines.
    - `anyhow`: For easy and robust error handling.
    - `dashmap`: For a high-performance, thread-safe hash map for data aggregation.

## Step 3: Initial Structure

- Created the initial command-line structure using `clap`.
- Set up a `main` function to handle file processing.
- Implemented a basic parallel file reading loop using `rayon`.
- Each file is processed line-by-line to keep memory usage low.
- Added basic error handling for file processing.

## Step 4: Aggregation of Error Counts

- Implemented logic to count lines containing `[error]`.
- Used `dashmap::DashMap` to safely handle concurrent updates to the error count.
- Used the `regex` crate to identify error lines.
- The final count is printed after all files have been processed.

## Step 5: IP Address Counting

- Modified the tool to parse and count IP addresses from log files.
- Used a regular expression with a named capture group (`?P<ip>...`) to extract IP addresses.
- The results now show a list of unique IP addresses and their frequencies.

## Step 6: Future Work

- Created a `TODO.md` file to document potential future enhancements.

## Step 7: Project Documentation

- Created a `README.md` file with a project overview, usage instructions, and a build guide.
- Created a `LICENSE` file containing the MIT License.
