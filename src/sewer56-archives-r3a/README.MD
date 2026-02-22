# sewer56-archives-r3a

[![Crates.io](https://img.shields.io/crates/v/sewer56-archives-r3a.svg)](https://crates.io/crates/sewer56-archives-r3a)
[![Docs.rs](https://docs.rs/sewer56-archives-r3a/badge.svg)](https://docs.rs/sewer56-archives-r3a)
[![CI](https://github.com/Sewer56/sewer56-archives-r3a/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/sewer56-archives-r3a/actions)

High Performance Archive Format for Mod Assets

## About

`sewer56-archives-r3a` is a modern, efficient archive format optimized for use in games and modding applications. It provides:

- **High-performance compression** with support for multiple algorithms (LZ4, Zstandard, BZip3)
- **SIMD optimizations** for critical operations
- **no_std compatibility** for embedded and bare-metal environments
- **Efficient deduplication** to minimize archive sizes
- **Flexible API** with builder pattern for easy integration

## Features

- `lz4` - Enables LZ4 compression/decompression support (enabled by default)
- `bzip3` - Enables BZip3 compression/decompression support (enabled by default)
- `hardened` - Adds additional runtime checks for untrusted input (enabled by default)
- `detect_num_cores` - Auto-detects system core count for parallel operations (enabled by default)
- `nightly` - Enables nightly-only optimizations
- `no_format` - Reduces binary size by avoiding core::fmt
- `pgo` - Profile-Guided Optimization support for benchmarks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sewer56-archives-r3a = "0.1.0"
```

For minimal binary size:

```toml
[dependencies]
sewer56-archives-r3a = { version = "0.1.0", default-features = false }
```

## Usage

### Creating an Archive

```rust,no_run
use sewer56_archives_r3a::api::packer_builder::*;

// Create and configure an archive builder
let mut builder = R3APackerBuilder::new()
    .with_solid_size(1048576)  // 1MB solid blocks
    .with_chunk_size(4194304)  // 4MB chunks
    .with_chunked_deduplication(true);

// Add files with custom options
builder.add_file_from_byte_slice(
    b"Hello, World!",
    AddFileParams::new("hello.txt".into())
);

// Building/serialization functionality is coming soon
// See the documentation for the latest API
```

### Extracting from an Archive

```text
// Archive extraction functionality is coming soon
// See the documentation for the latest API
```

## Documentation

For complete documentation, examples, and the format specification, visit:

- [Full Documentation](https://sewer56.dev/sewer56-archives-r3a/)
- [API Reference](https://docs.rs/sewer56-archives-r3a)
- [GitHub Repository](https://github.com/Sewer56/sewer56-archives-r3a)

## Contributing

See [CONTRIBUTING.MD](https://github.com/Sewer56/sewer56-archives-r3a/blob/main/CONTRIBUTING.MD) for guidance on how to contribute to this project.

## License

Licensed under [GPL v3 (with Reloaded FAQ)](https://github.com/Sewer56/sewer56-archives-r3a/blob/main/LICENSE).

For custom/alternative/commercial licensing options, contact admin@sewer56.dev
