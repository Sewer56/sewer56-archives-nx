# Rules for sewer56-archives-nx

## Project Overview

This is a modern, efficient archive format optimized for use in games. The library provides high-performance compression support, SIMD optimizations, and no_std compatibility.

## Project Structure

The project is organized into several main directories:

- **`/projects/sewer56-archives-nx/`** - Houses the main library implementation
  - `src/api/` - Public API interfaces and traits
  - `src/headers/` - Archive format headers and parsers
  - `src/implementation/` - Core implementation logic
  - `src/utilities/` - Utility functions and helpers
  - `benches/` - Performance benchmarks using criterion

- **`/projects/research/`** - Research tools and experiments (not production ready, research only)
  - Dictionary compression analysis tools
  - Mod archive statistics gathering
  - Compression algorithm comparisons

- **`/docs/`** - Documentation and specifications
  - Archive format specifications
  - Implementation details
  - Research findings

## Code Style & Formatting

### Rust Formatting

- Use proper rustdoc format with elements in brackets like [`Type`] instead of `Type`
- Maintain original code order and all comments intact unless explicitly permitted
- Try not to exceed 500 lines per module (excluding tests); split larger modules when necessary

### Variable Names & Structure

- Preserve existing coding style: keep variable names and loop structures unchanged unless explicitly instructed
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use descriptive names for performance-critical code

### Import and Dependency Preferences

- Prefer `core` over `std` when possible for better no_std compatibility
- Prefer using short names and `use` statements at the top of the file
- Only place `use` statements inside functions with conditional compilation flags like `#[cfg(...)]`

## Documentation Standards

### Function Documentation

- Use comprehensive rustdoc comments `///` for all public functions
- Include detailed Safety sections for unsafe functions covering:
  - Pointer validity requirements
  - Memory alignment recommendations
  - Buffer overlap restrictions
  - Size requirements and divisibility constraints
- Include Parameters and Returns sections
- Add Remarks section for complex behaviors or performance notes

### Examples

- Include code examples in documentation when helpful
- Show both basic usage and safety requirements

### Error Handling

- Maintain consistent error types across modules

## Backwards Compatibility

- The crates are not yet released; therefore, you may make backwards incompatible changes. Do not however rename methods unless it is necessary.

## Post-Change Verification

**CRITICAL: After making any code changes, ALWAYS perform these verification steps in order:**

1. **Run Tests**: Execute `cargo test --all-features` to ensure all functionality works correctly. Pass the `--workspace` flag if making changes to anything in `/projects/research/`.
2. **Auto-fix Clippy Issues**: Run `cargo clippy --workspace --fix --allow-dirty --all-features` to automatically fix linting issues
3. **Check Remaining Lints**: Run `cargo clippy --workspace --all-features -- -D warnings` to catch any remaining warnings
4. **Verify Documentation**: Run `cargo doc --workspace --all-features --no-deps` to check for documentation errors
5. **Fix Documentation Links**: For any broken doc links, use the proper format: `` [`function_name`]: crate::function_name ``
6. **Big Endian Testing** If `cross` is installed, run:
   ```
   cross test --package sewer56-archives-nx --target powerpc64-unknown-linux-gnu
   ```
7. **Format Code**: Run `cargo fmt --all` as the final step to ensure consistent formatting

These steps are mandatory and must be completed successfully before considering any code change complete.