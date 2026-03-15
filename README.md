# satelite-tle

A Rust library for parsing Satellite Two-Line Element (TLE) sets.

## Features

- Parse standard TLE format (3-line format including name).
- Support for parsing multiple satellites from a single string.
- Serde integration for easy serialization/deserialization.
Lightweight with minimal dependencies.

## Usage
- src/lib.rs unit test
- examples/read_beidou.rs  (run via `cargo run --example read_beidou`)