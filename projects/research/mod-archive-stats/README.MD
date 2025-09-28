# Mod Archive Statistics

A research project analyzing how people publish Reloaded-II mods by examining compression and file organization patterns across mod packages (~2500 at time of writing).

This was mostly written by LLM for research purposes. Not production ready code.

## Purpose

**Research Goal:** Understand how Reloaded-II mods are published and distributed to optimize archive formats.

1. **Extract data** - Download and analyze mod archives to generate file statistics
2. **Store results** - Output as `mod-stats.json.zst` (zstd compressed JSON) 
3. **Calculate statistics** - Analyze results from compressed data (tool not yet implemented)

## Data Source

Processes **Reloaded-II Mods** from the official mod index (~2500 at time of writing from GameBanana and other sources).

## Project Structure

```
mod-archive-stats/
├── make-mod-stats/          # CLI tool for data extraction
│   ├── src/commands/        # Pipeline stages
│   └── Cargo.toml
├── analyze-mod-stats/       # Analysis tool (not yet implemented)
└── mod-stats.json.zst      # Compressed results
```

## How It Works

Sequential three-stage pipeline processing mods **one by one**:

1. **Parse packages** - Downloads Reloaded-II mod index
2. **Download & extract** - Downloads, extracts, analyzes files, cleans up immediately  
3. **Generate statistics** - Compiles results into compressed output

**Sequential Processing:** Minimizes disk usage and respects servers.

## Usage

```bash
cd make-mod-stats
cargo run
```

Results automatically saved to `../mod-stats.json.zst`.

## Output Format

ZStandard-compressed JSON containing:
- Per-mod metadata with file-level statistics
- Each file includes: path, original size, compressed size (ZStd -16), XXH3 hash
- Overall summary with totals and error counts

## Dependencies

**Required:** 7-Zip command-line tool in PATH
- Ubuntu/Debian: `sudo apt install p7zip-full`
- macOS: `brew install p7zip` 
- Windows: Install 7-Zip, ensure `7z.exe` in PATH

## Testing

```bash
cargo test --features research-tests
```