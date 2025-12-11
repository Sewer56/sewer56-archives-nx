# Rules for sewer56-archives-nx

High-performance archive format for games with SIMD optimizations and `no_std` support.

## Structure (`sewer56-archives-nx/`)

- `src/api/` - Public API interfaces and traits
- `src/headers/` - Archive format headers and parsers
- `src/implementation/` - Core implementation logic
- `src/utilities/` - Utility functions and helpers
- `benches/` - Performance benchmarks (criterion)

## Code Style

- Rustdoc: use [`Type`] not `Type`
- Preserve original code order, comments, variable names, and loop structures unless explicitly permitted
- Max ~500 lines per module (excluding tests)
- Rust conventions: `snake_case` for functions/variables, `PascalCase` for types
- Prefer `core` over `std` for `no_std` compatibility
- `use` statements at file top; only inline with `#[cfg(...)]`

## Documentation

- `///` rustdoc for all public functions
- Unsafe functions require Safety section: pointer validity, alignment, buffer overlap, size constraints
- Include Parameters, Returns, and Remarks (for complex behaviors)
- Code examples when helpful

## Compatibility

Not yet released; breaking changes allowed, but avoid unnecessary method renames.

## Post-Change Verification

```bash
cargo test --all-features -q
cargo clippy --workspace --fix --allow-dirty --all-features
cargo clippy --workspace --all-features -q -- -D warnings
cargo doc --workspace --all-features --no-deps -q
cross test --package sewer56-archives-nx --target powerpc64-unknown-linux-gnu
cargo fmt --all
```
