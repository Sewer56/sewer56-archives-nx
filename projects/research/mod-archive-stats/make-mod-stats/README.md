# Make Mod Stats

A research tool for analyzing mod archive statistics from the Reloaded-II mod index. This tool downloads, extracts, and analyzes mod archives to generate comprehensive file statistics including original sizes, compression ratios, and content hashes.

This was mostly written by LLM, as a one-off for research. Not production ready code.

## Purpose

This tool measures detailed statistics for mod archives including:
- **File counts** - Total number of files per mod and across all mods
- **File sizes** - Original uncompressed file sizes 
- **Compressed sizes** - ZStandard level 16 compression ratios for each file
- **Content hashes** - XXH3 checksums for file deduplication analysis
- **Archive metadata** - Original download sizes and mod information

## Data Source

Processes **Reloaded-II Mods** from the official mod index (~2500 packages from GameBanana and other sources).

## Dependencies

**Required:** 7-Zip command-line tool must be available in PATH
- Install `7z` or `7zz` command-line tool
- On Ubuntu/Debian: `sudo apt install p7zip-full`
- On macOS: `brew install p7zip`
- On Windows: Install 7-Zip and ensure `7z.exe` is in PATH

## How It Works

The tool implements a sequential three-stage pipeline that processes mods **one by one**:

1. **Parse packages** - Downloads and parses `AllPackages.json.br` from Reloaded-II index
2. **Download & extract** - Downloads each mod archive individually, extracts using 7z, analyzes files, then immediately cleans up extracted content
3. **Generate statistics** - Compiles all analysis results into compressed output file

**Sequential Processing:** Mods are downloaded and processed one at a time to minimize disk space usage and be respectful to servers.

## Usage

```bash
cargo run -p make-mod-stats
```

The tool automatically runs through all pipeline stages and outputs compressed results to `../mod-stats.json.zst`.

## Output Format

Results are saved as ZStandard-compressed JSON containing:
- Per-mod metadata with file-level statistics
- Overall summary with totals and error counts
- Each file includes: path, original size, compressed size, XXH3 hash

## Archive Support

Supports all archive formats handled by 7-Zip including ZIP, 7Z, TAR, RAR, and others.

## Testing

```bash
cargo test -p make-mod-stats --features research-tests
```