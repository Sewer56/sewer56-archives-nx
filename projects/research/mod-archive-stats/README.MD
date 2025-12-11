# Mod Archive Statistics

Ever wondered how modders actually package their mods? This project downloads all mods from the [Reloaded-II.Index](https://github.com/Reloaded-Project/Reloaded-II.Index) (~2,200 unique packages at time of writing) and crunches the numbers on file sizes, compression ratios, and archive structures.

This was mostly written by LLM for research purposes. Not production ready code.

The analysis results are documented in the blog post: [Analyzing 2,200 Mods to Design an Archive Format For Modding](https://sewer56.dev/blog/2025/11/22/analyzing-2200-mods-to-design-an-archive-format-for-modding.html)

## Project Structure

```
mod-archive-stats/
├── make-mod-stats/          # Data collection tool
│   └── src/commands/        # Pipeline stages
├── analyze-mod-stats/       # Analysis & visualization tool
│   └── src/analysis/        # Analysis modules
├── mod-stats.json.zst       # Compressed results (download or generate)
└── README.MD                # This file
```

## Data Source

Processes **Reloaded-II Mods** from the official mod index (~2,200 unique packages at time of writing). The total Reloaded-II mod count is estimated at ~3,500, but the index did not track Nexus Mods at the time.

## Dependencies

**7-Zip** (for `make-mod-stats`):

- Ubuntu/Debian: `sudo apt install p7zip-full`
- macOS: `brew install p7zip`
- Windows: Install 7-Zip and ensure `7z.exe` is in PATH

**fontconfig** (for `analyze-mod-stats`, required by plotters):

- Ubuntu/Debian: `sudo apt-get install libfontconfig1-dev`
- Fedora/RHEL: `sudo dnf install fontconfig-devel`
- macOS: `brew install fontconfig`
- Windows: Not required (plotters uses GDI)

---

## make-mod-stats

Data collection tool that downloads, extracts, and analyzes mod archives to generate comprehensive file statistics.

### Purpose

Measures detailed statistics for mod archives including:

- **File counts** - Total number of files per mod and across all mods
- **File sizes** - Original uncompressed file sizes
- **Compressed sizes** - ZStandard level 16 compression ratios for each file
- **Content hashes** - XXH3 checksums for file deduplication analysis
- **Archive metadata** - Original download sizes and mod information

### How It Works

Sequential three-stage pipeline processing mods **one by one**:

1. **Parse packages** - Downloads and parses `AllPackages.json.br` from Reloaded-II index
2. **Download & extract** - Downloads each mod archive, extracts using 7z, analyzes files, then immediately cleans up
3. **Generate statistics** - Compiles all results into compressed output file

**Sequential Processing:** Mods are downloaded and processed one at a time to minimize disk space usage and be respectful to servers.

### Usage

```bash
cd make-mod-stats
cargo run --release
```

Results saved to this directory as `mod-stats.json.zst`.

Running this is expensive (~86 GB download at time of writing). Pre-computed data is available at [mod-stats+analysis.7z](https://sewer56.dev/blog/assets/mod-stats%2Banalysis.7z) - extract to this directory.

### Output Format

The file is ZStandard-compressed JSON containing:

- Per-mod metadata with file-level statistics
- Overall summary with totals and error counts
- Each file entry includes: path, original size, compressed size (ZStd -16), XXH3 hash

### Archive Support

Supports all archive formats handled by 7-Zip including ZIP, 7Z, TAR, RAR, and others.

---

## analyze-mod-stats

Analysis and visualization tool that processes the collected statistics to generate insights and plots.

### Purpose

Generates statistical analysis and visualizations including:

- **Archive size analysis** - Distribution of mod archive sizes
- **File count analysis** - Number of files per archive
- **File size analysis** - Individual file size distributions and compression ratios
- **Cumulative distribution plots** - Visual representations of the data

### How It Works

1. Reads `mod-stats.json.zst` from the parent directory (`mod-archive-stats/`)
2. Generates analysis outputs and plots to the same directory
3. Produces both text statistics and PNG plot files

### Usage

First, ensure `mod-stats.json.zst` exists in the `mod-archive-stats/` directory (either download pre-computed data or run `make-mod-stats`).

```bash
cd analyze-mod-stats
cargo run --release
```

### Output

Generates the following in the `mod-archive-stats/` directory:

- `archive_size_cumulative.png` - Cumulative distribution of archive sizes
- `file_count_cumulative.png` - Cumulative distribution of files per archive
- `compressed_file_sizes_cumulative.png` - Cumulative distribution of compressed file sizes
- Console output with detailed statistics and tables

---

## Testing

```bash
# Test make-mod-stats
cargo test -p make-mod-stats --features research-tests

# Test analyze-mod-stats
cargo test -p analyze-mod-stats
```

---

## Plot Conventions

The analysis tool generates cumulative distribution plots with specific unit conventions:

**X-axis Labels:** Use base-10 (decimal) units for consistency

- 1 kB = 1,000 bytes
- 1 MB = 1,000,000 bytes
- 1 GB = 1,000,000,000 bytes
- 1 TB = 1,000,000,000,000 bytes

**Why base-10?** This follows the SI standard for decimal prefixes and ensures plot labels accurately represent the byte values being displayed.

**Note:** This applies only to plot X-axis labels. The underlying data remains in bytes, and console statistics output may use different formatting conventions.
