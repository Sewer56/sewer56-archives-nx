# Format Specification

!!! tip "File Format Version: `1.0.0`"

!!! note

    This is a semi-SOLID archive format for storing game mod content; intended to double up as a packaging format for uploading mods.

It has the following properties:

- Files under Block Size are SOLID Compressed.
- Files above Block Size are non-SOLID Compressed.
- Variable Block Size.
- Stores File Hashes Within.
- Huge Files Split into Chunks for Faster (De)compression.
- TOC in-front.

We use SOLID compression to bundle up small files together, while keeping the large files as separate compressed blobs.
All files are entirely contained within a slice of a given block.

```mermaid
flowchart TD
    subgraph Block 2
        BigFile1.bin
    end

    subgraph Block 1
        BigFile0.bin
    end

    subgraph Block 0
        ModConfig.json -.-> Updates.json
        Updates.json -.-> more["... more .json files"]
    end
```

Offsets of each block is stored in header, therefore large files can be completely skipped during the extract operation
if a small file is all that is needed.

!!! note

    This format is optimized for transferring and unpacking files; editing existing archives might lead to sub-optimal performance.

## Overall Format Layout

The overall file is structured in this order:

```
| Header + TOC | Block 1 | Block 2 | ... | Block N |
```

All sections (indicated by `|`) are 4096 aligned to match physical sector size of modern drives and page granularity.

Field sizes used below are similar to Rust notation; with some custom types e.g.

- `u8`: Unsigned 8 bits.
- `i8`: Signed 8 bits.
- `u4`: 4 bits.
- `u32/u64`: 4 Bytes or 8 Bytes (depending on variant).
- `align8`: Add 0-7 padding bytes to align the current address to multiple of 8.

Assume any bit packed values are sequential, i.e. if `u4` then `u4` is specified, first `u4` is the upper 4 bits.

All packed fields are `little-endian`; and written out when total number of bits aligns with a power of 2.

- `u6` + `u12` is 2 bytes `little-endian`
- `u15` + `u17` is 4 bytes `little-endian`
- `u26` + `u22` + `u16` is 8 bytes `little-endian`
- `u6` + `u11` + `u17` ***is 4 bytes*** `little-endian`, ***not 2+2***

### Terminology

- `Block`: Represents a compressed section of data of any size smaller than [chunk size](./File-Header.md#chunk-size).
- `Chunk`: A `block` that corresponds to a slice of a file.
    - A file compressed in a single `block` is said to have 1 chunk.
    - A file compressed in multiple `block`(s) is said to have multiple chunks.

## Use as Packaging Format

!!! tip

    Inclusion of hash for each file has some nice benefits.

- Can do partial download to upgrade from older version of mod.
    - We can download header (incl. [Table of Contents][ToC Header]) only, compare hashes.
    - Then only download the chunks we need to decompress our needed data.
    - Inspired by MSIX and certain Linux package formats.

- Certain applications like [Nexus Mods App] can avoid re-hashing files.

## Previewing the Format

!!! info

    For people wishing to study the format, or debug it, a [010-Editor](https://www.sweetscape.com/010editor/) template
    is available for usage [010 Template](./010Template.bt).

Hit `Templates -> Open Template` and then the big play button.
Then you'll be able to browse the format in 'Variables' window.

Alternatively, contributions are welcome if anyone wants to make a [Kaitai Struct](https://kaitai.io) variation 💜.

## Section Alignment

!!! info "Each section is aligned to the following values in bytes"

    For arrays, this lists alignment for each entry.

- [File Header][File Header]: 8

Table of Contents:

- [ToC Header][ToC Header]: 8
- [FileEntry[FileCount]][FileEntry]: 4 (V0) / 8 (V1)
- [Blocks[BlockCount]][Blocks]: 4
- [StringPool][StringPool]: 4

User Data:

- `align8`: 0-7
- [User Data Header][User Data Header]: 8

File entries are aligned to 8 bytes when [Version] is V0,

## Version History

!!! info "This is the version history for the file format, not the reference implementation/library."

To view the file format specification for a given version, navigate to the linked commit
for each version and read this specification.

### 2.0.0

!!! info "Initial Release, in Rust"

    Version in header is updated to 1.

***THIS IS A WIP. REST OF SPEC IS NOT YET UPDATED TO ACCOUNT FOR THIS***

- Hashing algorithm replaced with [XXH3] (from [XXH64][XXH3]).
- Added support for new 'String Pool' format.
- [Unconfirmed] Support for per-extension dictionaries.
- Implementation of User Data Segment in reference implementation.
- Added `Section Alignment` section to docs.

#### Implementation of User Data Segment

!!! info "The `User Data Segment`, proposed in 1.X docs is finalized and implemented."

Example use cases:

- Storing a binary baked-in hashtable to quickly find files by name.
- Storing update information for a mod package if Nx is used to power a package manager.
- Storing file metadata (read/write timestamps, file permissions, etc.)

#### Implementation of Per-Extension Dictionaries

See: [Per-Group Dictionary Experiment](https://github.com/Sewer56/sewer56-archives-nx/issues/1)

#### Hashing Algorithm Change

The hashing algorithm has been changed to [XXH3] from [XXH64][XXH3].

This is a hard change because [XXH3] is superior in just about all use cases.
The format originally intended to use [XXH3], however the [Nexus Mods App] opted
to go with [XXH64][XXH3] instead.

The original intent was that you'd take the hash of each file from the archive and get hashes
'for free' (no I/O bottleneck). However the design changed.

Since the [Nexus Mods App] does not make use of the hashes in the archives, the archive
format is migrating to [XXH3] as standard.

#### String Pool

The format of the [String Pool] was slightly modified in order to speed up parsing the archive headers.
The string pool now starts with an array of `u8` with the path lengths. The strings follow after this.

This speeds up parsing the string pool.

### 1.1.0

!!! info "Revisions of the Spec"

    This is a minor revision of the spec which tightens some assumptions about the format.

This does not increment the version in the header. There are no changes in the actual format itself,
just that certain behaviours of the reference implementation are being standardised into the spec.

Version in header remains 0.

#### Tightened String Pool Assumptions

The [String Pool] is now assumed to have a number of items equivalent to the amount
of files which are stored in the Table of Contents. In other words, [FileCount] == `NumOfItemsInPool`.

This was already the case previously, but now this is part of the spec.

!!! tip "This allows for faster parsing of pool"

### 1.0.0

!!! info "Initial Release"

    Last commit with previous version: [196d116d09cd436818dfd596e069eaef2b7a616d](https://github.com/Nexus-Mods/NexusMods.Archives.Nx/commit/196d116d09cd436818dfd596e069eaef2b7a616d)

Dated 21st of July 2024, this marks the 'initial release' as `1.0.0`.

#### File Header Changes

This release removes the `Block Size` (u4) field from the header, as this can vary
per block and with the use of features such as deduplication and archive merging.
It was also not used in the reference implementation anywhere.

Instead, the `Chunk Size` field is extended to 5 bits and the header page count to
15 bits. This allows the [chunk size](./File-Header.md#chunk-size) to be in range
of `512 bytes` to `1 TiB`. (Previous range `32K` - `1GiB`)

The version field is repurposed. In the previous version, it was used to indicate
the version of the table of contents. Now that is moved to the actual table
of contents itself. The version field is now used to indicate incompatible changes
in the format itself. This field is `u7`. The previous field, was moved to the actual
[Table of Contents](./Table-Of-Contents.md#version) itself.

The `Header Page Count` field is extended to 16 bits, allowing for a max size of
256MiB. This allows for storage of [arbitrary user data][User Data Header]
as part of the Nx header. A reserved, but not yet implemented section for
[User Data][User Data Header] was also added to the header.

The [Table of Contents][ToC Header] has also received its own proper
'size' field. Which led to some fields being slightly re-organised.


[String Pool]: ./Table-Of-Contents.md#string-pool
[FileCount]: ./Table-Of-Contents.md#file-count
[XXH3]: https://xxhash.com/
[Nexus Mods App]: https://github.com/Nexus-Mods/NexusMods.App
[FileEntry]: ./Table-Of-Contents.md#file-entries
[ToC Header]: ./Table-Of-Contents.md
[Blocks]: ./Table-Of-Contents.md
[StringPool]: ./Table-Of-Contents.md#string-pool
[File Header]: ./File-Header.md
[User Data Header]: ./User-Data.md