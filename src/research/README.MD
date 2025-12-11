# Research Tools

A collection of tools for researching compression algorithms, dictionary effectiveness, and archive format optimization. Use these to analyze how different compression strategies perform on your data.

**Disclaimer:** These tools are largely AI Slop, mostly made by Sonnet 3.5. This is rough stuff, not production quality code.

## Quick Reference

| Tool                                | Language | Description                                                                                      |
| ----------------------------------- | -------- | ------------------------------------------------------------------------------------------------ |
| `benchmark-single-file-dict.py`     | Python   | Benchmarks ZSTD dictionary compression by splitting files into blocks and comparing performance  |
| `benchmark-dict-over-extension.py`  | Python   | Compares individual vs solid block compression with/without dictionaries for specific extensions |
| `test-single-file-dict-size.py`     | Python   | Analyzes dictionary compression effectiveness on single large files with detailed statistics     |
| `compare-compression-algorithms.py` | Python   | Benchmarks multiple compression algorithms (7z, bzip3, zstd, etc.) across directories            |
| `compare-best-between-folders.py`   | Python   | Compares file sizes across multiple directories to find optimal compression methods              |
| `common.py`                         | Python   | Shared utility functions used by other Python scripts                                            |
| `dictionary-tester/`                | Rust     | High-performance analyzer for dictionary compression                                             |
| `mod-archive-stats/`                | Rust     | Analyzes compression patterns across Reloaded-II mod archives                                    |

## Getting Started

This directory is a **separate Cargo workspace** from the main `sewer56-archives-nx` library to avoid imposing additional system dependencies on library users.

**Build Rust tools:**

```bash
cd src/research # directory with this README file
cargo build --release
```

> **Note:** If the build fails, you may need to install [system dependencies](#system-requirements).

**Run Python scripts:**

```bash
cd src/research # directory with this README file
python3 <script-name>.py --help
```

> **Note:** Python tools require `zstd` CLI. See [Python prerequisites](#prerequisites).

---

## Python Tools

### Prerequisites

All Python tools require:
- **Python 3.x** with standard library modules
- **`zstd` CLI** - ZStandard compression tool

Some tools require additional compressors (noted in each tool's section).

---

### 1. `benchmark-single-file-dict.py`

Benchmarks ZSTD dictionary compression by splitting a file into blocks and comparing compression performance with and without dictionaries.

**Sample Usage:**

```bash
python3 benchmark-single-file-dict.py /path/to/test_files --extension txt
python3 benchmark-single-file-dict.py /path/to/test_files -e json --fixed-dict-size
python3 benchmark-single-file-dict.py /path/to/test_files -e bin --dict-size 65536
```

**Output:**

```text
=== Benchmark Summary ===
Files processed: 7

large_file.txt:
  Average speed without dict: 2471.62 MB/s
  Average speed with dict: 4262.35 MB/s
  Average difference: +1790.74 MB/s (+72.5%)

big_text_0.txt:
  Average speed without dict: 3446.48 MB/s
  Average speed with dict: 9641.49 MB/s
  Average difference: +6195.01 MB/s (+179.7%)

Overall averages:
  Without dictionary: 3249.63 MB/s
  With dictionary: 8608.25 MB/s
  Difference: +5358.62 MB/s (+164.9%)
```

**Options:**

| Option              | Description                                                |
| ------------------- | ---------------------------------------------------------- |
| `--block-size`      | Block size in KiB (default: 128)                           |
| `--fixed-dict-size` | Use fixed 110 KiB dictionary instead of 1/100 of file size |
| `--dict-size`       | Manually specify dictionary size in bytes                  |
| `--extension`       | Filter by specific file extension                          |

---

### 2. `benchmark-dict-over-extension.py`

Comprehensive benchmark tool that compares individual file compression vs solid block compression, both with and without dictionaries, for files of a specific extension.

**Sample Usage:**

```bash
python3 benchmark-dict-over-extension.py /path/to/files --extension txt
python3 benchmark-dict-over-extension.py /path/to/files -e json --mode trim --block-size 131072
python3 benchmark-dict-over-extension.py /path/to/files -e bin --mode small --dict-size 110000
```

**Output:**

```text
=== Compression Summary ===
Total original size: 16.00 KiB

Compressed sizes:
  Individual files (no dict):   4.39 KiB (3.64x)
  Individual files (with dict):  1.63 KiB (9.83x)
  Solid blocks (no dict):        4.39 KiB (3.64x)
  Solid blocks (with dict):      1.63 KiB (9.83x)

Space savings vs individual (no dict):
  Dictionary advantage:         2.76 KiB (62.9%)
  Solid block advantage:        0.00 B (0.0%)
  Solid block + dict advantage: 2.76 KiB (62.9%)

Average decompression speeds:
  Individual files (no dict):   1211.01 MB/s
  Individual files (with dict):  4334.01 MB/s
  Solid blocks (no dict):        1208.01 MB/s
  Solid blocks (with dict):      4546.14 MB/s
```

**Options:**

| Option         | Description                                                          |
| -------------- | -------------------------------------------------------------------- |
| `--mode`       | Choose 'small' for files under size limit or 'trim' for larger files |
| `--block-size` | Block size in bytes (default: 64KB)                                  |
| `--dict-size`  | Dictionary size in bytes (default: 110KB)                            |
| `--extension`  | File extension to process (required)                                 |

---

### 3. `test-single-file-dict-size.py`

Analyzes dictionary compression effectiveness on a single large file by splitting it into blocks and providing detailed per-block statistics.

**Sample Usage:**

```bash
python3 test-single-file-dict-size.py /path/to/large_file.bin
python3 test-single-file-dict-size.py /path/to/large_file.bin --fixed-dict-size
python3 test-single-file-dict-size.py /path/to/large_file.bin --dict-size 131072 --block-size 256
```

**Output:**

```text
Processing test_data/large_file.txt...
Input file size: 85.40 KiB
Target dictionary size: 110.00 KiB (110 KiB (fixed))
Splitting into 8 KiB blocks...

Overall Size Comparison:
Original size:              114.95 KiB
Dictionary size:            36.96 KiB (32.2%)
Dictionary compressed:      6.74 KiB (5.9%)
Compressed (no dict):       13.50 KiB (11.7%)
Compressed (with dict):     9.31 KiB (8.1%)
Total with dict+blocks:     16.05 KiB (14.0%)

Overall Space Savings:
Without dictionary:         88.26%
With dictionary:            86.03%
Dictionary advantage:       -2.22%

Per-Block Statistics:
Number of blocks: 11
Blocks with >=4KiB improvement: 0 (0.0%)
Average 4KiB units saved per block: 0.00

Dictionary Advantage (percentage points):
  Min: 1.75%
  Max: 6.14%
  Avg: 4.84%

Most Improved Block:
  block_0009:
    Original: 8.00 KiB
    Without dict: 1.28 KiB (83.96% saved)
    With dict: 811.00 B (90.10% saved)
    Advantage: 6.14%
    Bytes saved: 503.00 B
```

**Options:**

| Option              | Description                                            |
| ------------------- | ------------------------------------------------------ |
| `--fixed-dict-size` | Use fixed 110 KiB dictionary instead of dynamic sizing |
| `--block-size`      | Block size in KiB (default: 128)                       |
| `--dict-size`       | Manual override for dictionary size in bytes           |

---

### 4. `compare-compression-algorithms.py`

Benchmarks multiple compression algorithms across an entire directory of files using parallel processing.

**Sample Usage:**

```bash
python3 compare-compression-algorithms.py /path/to/test_directory
python3 compare-compression-algorithms.py /path/to/test_directory 8  # Use 8 threads
```

**Output:**

```text
Original size: 1.21 MiB
Total files to process: 26
Starting compression benchmark...

Running bzip3 16m compression...
[Processing files...]

Running 7z compression...
[Processing files...]

=== Compression Benchmark Results ===
Input directory: test_data
Original size: 1.21 MiB
Files processed: 26
Threads used: 2

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
[Results depend on available compression tools]

Total benchmark time: 15.2s
Results stored in: /path/to/test_data_compressed_20240924_193504
```

**Additional Dependencies:**

This tool requires multiple compression utilities. Missing tools will be skipped.

| Tool    | Installation                                    |
| ------- | ----------------------------------------------- |
| `7z`    | `apt install p7zip-full` / `brew install p7zip` |
| `xz`    | Usually pre-installed on Linux/macOS            |
| `bzip3` | Build from source or package manager            |
| `kanzi` | <https://github.com/flanglet/kanzi-go>          |
| `pigz`  | `apt install pigz` / `brew install pigz`        |

**Tweaking:**

- Edit the `compressors` list in the script to add/remove algorithms
- Modify compression levels in the `compress_file()` function
- Adjust thread count via command line argument

---

### 5. `compare-best-between-folders.py`

Compares file sizes across multiple directories to find the best compression method for each file and calculate potential space savings.

**Sample Usage:**

```bash
python3 compare-best-between-folders.py /path/to/compressed_dir1 /path/to/compressed_dir2 /path/to/compressed_dir3
```

**Output:**

```text
File Size Comparisons:
=========================================
Path    folder1 folder2 Best    Savings %
-----------------------------------------
example 38.00B  61.00B  folder1 37.70%   
file2   18.00B  36.00B  folder1 50.00%   
=========================================

Folder Totals:
folder1: 56.00B
folder2: 97.00B

Best Combination Total: 56.00B
Potential Space Saving vs Worst: 41.00B
```

**Tweaking:**

- Modify `get_percent_diff()` to change percentage calculation
- Adjust table formatting in the main output loop
- Change file matching logic in `get_base_name()` for different naming conventions

---

### 6. `common.py`

Shared utility functions for the other Python scripts.

**Functions Include:**

- File splitting into blocks
- Dictionary training with zstd
- Block compression with/without dictionaries
- Size formatting and benchmarking utilities

---

## Rust Tools

### System Requirements

Building the Rust tools requires additional system dependencies:

**Linux (Debian/Ubuntu):**

```bash
# For fontconfig (required by plotters)
sudo apt-get install libfontconfig1-dev
```

**Linux (Fedora/RHEL):**

```bash
sudo dnf install fontconfig-devel
```

**macOS:**

```bash
brew install fontconfig
```

**Windows:**

- Fontconfig is typically not required (plotters uses GDI)

---

### `dictionary-tester/`

High-performance analyzer for dictionary compression effectiveness using the Nx archive format's native compression utilities.

**Sample Usage:**

```bash
cd dictionary-tester
cargo run --release --features nightly -- -i /path/to/test_directory
cargo run --release --features nightly -- -i /path/to/test_directory -d 131072 -l 19 -b 2097151
cargo run --release --features nightly -- -i /path/to/test_directory --no-solid-blocks
```

**Output:**

```text
Starting compression analysis:
Input directory: ../test_data
Block size: 1048575 bytes
Chunk size: 1048576 bytes
Compression level: 16
Dictionary size: 65536 bytes

Found 26 files, total size: 1.2 MiB

Breakdown by extension (group):
txt: 10 files, 639.9 KiB
json: 13 files, 598.8 KiB
(no extension): 3 files, 440 B

Analyzing compression by extension...
Processing txt files. Dict Content size 639.9 KiB Complete in 1.390839708s Dict size 41.3 KiB
Processing json files. Dict Content size 598.8 KiB Complete in 1.543960521s Dict size 58.0 KiB

Compression Analysis Results:

json
Original Size: 598.8 KiB
Compressed w/o Dict: 54.4 KiB
Compressed with Dict: 52.7 KiB
Improvement: 1.6 KiB
Average compression ratio: 9.08%
Average compression ratio with dictionary: 8.80%
Average improvement with dictionary: 1.6 KiB

txt
Original Size: 639.9 KiB
Compressed w/o Dict: 30.2 KiB
Compressed with Dict: 28.2 KiB
Improvement: 2.0 KiB
Average compression ratio: 4.72%
Average compression ratio with dictionary: 4.41%
Average improvement with dictionary: 2.0 KiB

Total Results:
Total Original Size: 1.2 MiB
Total Compressed w/o Dict: 84.7 KiB
Total Compressed with Dict: 81.1 KiB
Total Improvement: 3.7 KiB
Total Dict Size: 99.3 KiB
Overall Improvement Percentage: 0.30%
```

**Options:**

| Option                  | Description                                                   |
| ----------------------- | ------------------------------------------------------------- |
| `-b, --block-size`      | Maximum size of individual blocks in bytes (default: 1048575) |
| `-c, --chunk-size`      | Maximum chunk size in bytes (default: 1048576)                |
| `-l, --level`           | Compression level from -5 to 22 for zstd (default: 16)        |
| `-d, --dict-size`       | Dictionary size in bytes (default: 65536)                     |
| `-n, --no-solid-blocks` | Disable solid block compression                               |

**Requirements:**

- Rust toolchain with nightly features support
- Feature flag: `nightly` (required for allocation APIs)
- At least 7 sample files for dictionary training

---

### `mod-archive-stats/`

Analyzes compression and file organization patterns across Reloaded-II mod archives (~2,200 unique mods at time of writing). Contains two tools:

- **make-mod-stats** - Data collection tool that downloads and analyzes mod archives
- **analyze-mod-stats** - Analysis and visualization tool that generates statistics and plots

For full documentation, see [mod-archive-stats/README.MD](mod-archive-stats/README.MD).