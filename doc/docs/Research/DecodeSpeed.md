# Tuning Decode Speed

A page of notes/random rambling related to decode.

!!! info "For ZStandard, `v1.5.6` on 5900X"

!!! note "ZStandard has different settings for compression levels depending on input size"

    Namely for `> 256KiB`, `<= 256 KiB`, `<= 128 KiB` and `<= 16 KiB` respectively.

We're going to show a small number of samples.

These tests have been ran with other inputs, such as the Silesia corpus; so the results here
are fairly representative.

## Poorly Compressed Files

!!! info "Source: `Whiterun alternative street stone by Pfuscher 2.2-2347-2-2-1578423661.7z`"

    File: `wrstonefloor01_n.dds`
    Original size: 85.3MiB

We're using a 'normal map' texture for this test, since those tend to encode very minute details,
and thus are prone to contain a lot of randomness.

### Full File

```bash
# Compress
zstd -12 wrstonefloor01_n.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n.dds.zst

# 79.5 MiB, 1262.1 MB/s
```

```bash
# Compress
zstd -16 wrstonefloor01_n.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n.dds.zst

# 79.4 MiB, 1264.1 MB/s
```

Levels 17 and over slow down.

```bash
# Compress
zstd -19 wrstonefloor01_n.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n.dds.zst

# 70.3 MiB, 676.1 MB/s
```

### Full File (No Huffman)

```bash
# Compress
./compressor wrstonefloor01_n.dds wrstonefloor01_n.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_n.dds.nohuf.zst

# 80.6 MiB, 2972.5 MB/s
```

### Small Blocks (128K)

!!! info "First 128KiB of the above file"

```bash
# Compress
zstd -12 wrstonefloor01_n_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n_0_131072.dds.zst

# 120KiB, 1190.3 MB/s
```

```bash
# Compress
zstd -13 wrstonefloor01_n_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n_0_131072.dds.zst

# 120KiB, 1178.5 MB/s
```

```bash
# Compress
zstd -14 wrstonefloor01_n_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_n_0_131072.dds.zst

# 111KiB, 769.0 MB/s
```

### Small Blocks (No Huffman)

```bash
# Compress
./compressor wrstonefloor01_n_0_131072.dds wrstonefloor01_n_0_131072.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_n_0_131072.dds.nohuf.zst

# 123.6KiB, 4868.4 MB/s
```

## Well Compressed Files

!!! info "Source: `Whiterun alternative street stone by Pfuscher 2.2-2347-2-2-1578423661.7z`"

    File: `wrstonefloor01.dds`
    Original size: 64.0MiB

This is a texture of bricks, there are no rows of repeated pixels here, however the bricks themselves
are fairly similar. This is roughly a typical texture you'll find in a mod or a game.

### Full File

```bash
# Compress
zstd -12 wrstonefloor01.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01.dds.zst

# 31.6 MiB, 1008.4 MB/s
```

```bash
# Compress
zstd -16 wrstonefloor01.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01.dds.zst

# 30.2 MiB, 990.0 MB/s
```

Levels 19 and over don't slow down for well compressible.

```bash
# Compress
zstd -19 wrstonefloor01.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01.dds.zst

# 27.7 MiB, 976.7 MB/s
```

### Full File (No Huffman)

```bash
# Compress level 12
./compressor wrstonefloor01.dds wrstonefloor01.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01.dds.nohuf.zst

# 33.2 MiB, 1109.2 MB/s
```

1.6MiB bigger, and decode is 10% faster.

```bash
# Compress level 16
./compressor wrstonefloor01.dds wrstonefloor01.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01.dds.nohuf.zst

# 31.5 MiB, 1039.2 MB/s
```

1.3MiB bigger, and decode is ~5.5% faster.


### Small Blocks (128K)

!!! info "First 128KiB of the above file"

```bash
# Compress
zstd -12 wrstonefloor01_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.zst

# 72.1KiB, 963.0 MB/s
```

```bash
# Compress
zstd -13 wrstonefloor01_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.zst

# 70.3KiB, 889.0 MB/s
```

```bash
# Compress
zstd -16 wrstonefloor01_0_131072.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.zst

# 68.0KiB, 585.8 MB/s
```

### Small Blocks (128K) (No Huffman)

```bash
# Compress level 12
./compressor wrstonefloor01_0_131072.dds wrstonefloor01_0_131072.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.nohuf.zst

# 80.8KiB, 1239.1 MB/s
```

8.7KiB bigger, and decode is 29% faster.

```bash
# Compress level 3
./compressor wrstonefloor01_0_131072.dds wrstonefloor01_0_131072.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.nohuf.zst

# 97.0KiB, 1863.8 MB/s
```

With compression level 3, we get a neat speedup.
Decode is 1.8x faster at a bit of a loss in compression ratio.
But LZ4 is preferable here.

#### LZ4 For Comparison

```bash
lz4 -T1 -12 -b wrstonefloor01_0_131072.dds
# 131072 ->     91496 (1.433),  29.0 MB/s, 3153.7 MB/s
```

### Small Blocks (128K) (Dictionary)

Using a dictionary (110KiB) trained against 128KiB of the above file.

!!! note "The dictionary being trained on the blocks of the file itself"

```bash
# Compress with dictionary
zstd -12 -D trained_dict wrstonefloor01_0_131072.dds

# Benchmark
zstd -b -d -i20 -D trained_dict wrstonefloor01_0_131072.dds.zst

# 68.1KiB, 741.9 MB/s
```

Decompression slows down when using `-12`.

```bash
# Compress with dictionary
zstd -16 -D trained_dict wrstonefloor01_0_131072.dds

# Benchmark
zstd -b -d -i20 -D trained_dict wrstonefloor01_0_131072.dds.zst

# 65.1KiB, 667.7 MB/s
```

But speeds up compared to no dict by 1.1x at higher levels.

## Tricks

### Disabling Huffman

We're technically talking about 'entropy coding' here, rather than actually 'huffman', but the
term 'huffman' is better known, and used in zstd's own docs.

???+ note "Dummy C code to compress without a file without huffman literals"

    ```c

    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
    #define ZSTD_STATIC_LINKING_ONLY
    #include <zstd.h>
    #include <zstd_errors.h>

    #define MAX_BUFFER_SIZE (1 << 29)

    int compress_file(const char* input_path, const char* output_path) {
        FILE* input_file = fopen(input_path, "rb");
        FILE* output_file = fopen(output_path, "wb");

        if (!input_file || !output_file) {
            fprintf(stderr, "Error opening files\n");
            return 1;
        }

        // Allocate compression context
        ZSTD_CCtx* cctx = ZSTD_createCCtx();
        if (cctx == NULL) {
            fprintf(stderr, "ZSTD_createCCtx() failed\n");
            return 1;
        }

        // Set compression parameters
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_compressionLevel, 12);
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_literalCompressionMode, ZSTD_lcm_uncompressed);

        // Allocate memory for input and output buffers
        void* input_buffer = malloc(MAX_BUFFER_SIZE);
        void* output_buffer = malloc(ZSTD_compressBound(MAX_BUFFER_SIZE));

        if (!input_buffer || !output_buffer) {
            fprintf(stderr, "Memory allocation failed\n");
            return 1;
        }

        size_t total_input = 0;
        size_t total_output = 0;

        // Compression loop
        while (1) {
            size_t read_size = fread(input_buffer, 1, MAX_BUFFER_SIZE, input_file);
            if (read_size == 0) break;
            total_input += read_size;

            size_t compressed_size = ZSTD_compress2(cctx,
                                                    output_buffer, ZSTD_compressBound(read_size),
                                                    input_buffer, read_size);

            if (ZSTD_isError(compressed_size)) {
                fprintf(stderr, "Compression error: %s\n", ZSTD_getErrorName(compressed_size));
                return 1;
            }

            fwrite(output_buffer, 1, compressed_size, output_file);
            total_output += compressed_size;
        }

        // Clean up
        ZSTD_freeCCtx(cctx);
        fclose(input_file);
        fclose(output_file);
        free(input_buffer);
        free(output_buffer);

        printf("Original size: %zu bytes\n", total_input);
        printf("Compressed size: %zu bytes\n", total_output);
        printf("Compression ratio: %.2f%%\n", ((float)total_output / total_input) * 100);

        return 0;
    }

    int main(int argc, char** argv) {
        if (argc != 3) {
            fprintf(stderr, "Usage: %s <input_file> <output_file>\n", argv[0]);
            return 1;
        }

        return compress_file(argv[1], argv[2]);
    }
    ```

### Tuning Min Match Length

!!! info "A large cause of slowdown is the match length"

    `ZSTD_CCtx_setParameter(cctx, ZSTD_c_minMatch, *);` in the C API

This is the cause of the discrepancy between Level 13 and Level 14
in the various [128K block tests](#small-blocks-128k_1).

Running `level 16` with min match length of 4 (`ZSTD_CCtx_setParameter(cctx, ZSTD_c_minMatch, *);`)
yields the following results:

```bash
# Compress level 16, 4 min match length
./compressor wrstonefloor01_0_131072.dds wrstonefloor01_0_131072.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.nohuf.zst

# 70.1KiB, 892.3 MB/s
```

and without huffman:

```bash
# Compress level 16, 4 min match length
./compressor wrstonefloor01_0_131072.dds wrstonefloor01_0_131072.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_131072.dds.nohuf.zst

# 76.8KiB, 994.8 MB/s
```

#### Effect on Larger Blocks (1MB)

With default match length (5) for this size:

```bash
# Compress
zstd -16 --no-check wrstonefloor01_0_1M.dds

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_1M.dds.zst

# 553.0KiB, 1025.5 MB/s
```

With match length of 6:

```bash
# Compress level 16
./compressor wrstonefloor01_0_1M.dds wrstonefloor01_0_1M.dds.nohuf.zst

# Benchmark
zstd -b -d -i20 wrstonefloor01_0_1M.dds.nohuf.zst

# 575.2KiB, 1164.8 MB/s
```

A minimal regression in decode speed.
Same can be observed at lower levels.

### No Checksum

This applies only to zstd CLI, we don't hash compare the file in Nx unless requested.

```
zstd --no-check
```

When compressing with zstd, add `no-check` to the commandline, this will produce a file without a checksum.

A no-checksum file will be faster during benchmarking, by around 5%.

## Decompress Speed Targets

!!! info "How fast do we need to decode files?"

Let's first establish some performance numbers.
Nx (2.0+) will be able to easily saturate any non-NVMe drive, so we're jumping straight
into max read speeds (Q8 T1).

NVMe PCI-e 4.0:

- Entry Level: 4132 MB/s (SN580)
- High End: 6148 MB/s (NV3)
- Top End: 7130 MB/s (990 EVO Plus & Solidigm P44 Pro)

NVMe PCI-e 5.0:

- (Current) High End: 12393 MB/s

!!! note "At time of writing 5.0 drives are still relatively new"

### Takeaways

- Disabling huffman gives you a ~10% boost in decode speed, with results varying depending on level.
- Most effective way to speed up decompression is to increase min match length.

### Target Decompression Speed

!!! info "Since R3A is tuned for gaming, we're going to focus on the main sources of file size there"

To calculate the speed we need to decompress at, the formula is simple:

```
DriveSpeed * 1/CompressionRatio
```

So if `CompressionRatio` is 0.5 (files are half the size) and `DriveSpeed` is
6000 MB/s, we need to decompress at 12000 MB/s to keep up.

#### Textures

Textures.

For this I tested with `-Skyrim 202X 10.0.1 - Architecture PART 1-2347-10-0-1710488193`.
I removed all non regular textures, i.e. those ending with `_n.dds` and `_p.dds` as most game mods
don't ship normals etc.

Original dataset is 9.48GiB

| Level | Size    | Ratio  |
| ----- | ------- | ------ |
| 12    | 6.79GiB | 0.7166 |
| 16    | 6.73GiB | 0.7093 |

Required decompression speed to saturate I/O:

| Level | 4.0 Entry Level | 4.0 Top End | 5.0 Current |
| ----- | --------------- | ----------- | ----------- |
| 12    | 5766 MB/s       | 9950 MB/s   | 17294 MB/s  |
| 16    | 5825 MB/s       | 10052 MB/s  | 17472 MB/s  |

***With 1M blocks (zstd cli):***

| Level | Size    | Ratio  |
| ----- | ------- | ------ |
| 12    | 6.92GiB | 0.7302 |

Difference is around 2%.

***With 1M blocks (original Nx library)***:

Including header info, 100% independent blocks and file padding.

| Level | Size   | Ratio |
| ----- | ------ | ----- |
| 12    | 7.2GiB | 0.759 |

Difference is around 2%.

### Reference Numbers

Speeds vary with block size, and depending on whether we use long mode for matching.

For simplicity, I have [reuploaded the data set mentioned above][bench-dataset].

To benchmark use original Nx CLI, example below:

```
dotnet ./cli/NexusMods.Archives.Nx.Cli.dll benchmark --source out-1M.nx --threads 12
```

!!! note "On a scale of 1-10, this is a '7' in terms of decompression speed."

    As far as this dataset is concerned.

    Worst case scenarios (1), we can expect decompression around ~20% slower than this.

#### 1M Chunks

```
dotnet ./cli/NexusMods.Archives.Nx.Cli.dll pack --source "textures" --target "out-1M.nx" --solidlevel 16 --chunkedlevel 16 --chunksize 1048576
```

AMD Ryzen 9 5900X, 32GB DDR4-3200 (16-17-17-35)

| Threads | Speed (GiB/s) |
| ------- | ------------- |
| 1       | ~1.27 GiB/s   |
| 2       | ~2.52 GiB/s   |
| 3       | ~3.73 GiB/s   |
| 4       | ~4.93 GiB/s   |
| 6       | ~7.22 GiB/s   |
| 8       | ~9.02 GiB/s   |
| 12      | ~11.74 GiB/s  |
| 24      | ~12.00 GiB/s  |

#### 1M Chunks, Long Mode

```
dotnet ./cli/NexusMods.Archives.Nx.Cli.dll pack --source "textures" --target "out-1M-long.nx" --solidlevel 22 --chunkedlevel 22  --chunksize 1048576
```

AMD Ryzen 9 5900X, 32GB DDR4-3200 (16-17-17-35)

| Threads | Speed (GiB/s) |
| ------- | ------------- |
| 1       | ~1.04 GiB/s   |
| 2       | ~2.04 GiB/s   |
| 3       | ~3.02 GiB/s   |
| 4       | ~3.96 GiB/s   |
| 6       | ~5.89 GiB/s   |
| 8       | ~7.49 GiB/s   |
| 12      | ~10.65 GiB/s  |
| 24      | ~11.49 GiB/s  |

#### 16M Chunks

```
dotnet ./cli/NexusMods.Archives.Nx.Cli.dll pack --source "textures" --target "out-16M.nx" --solidlevel 16 --chunkedlevel 16 --chunksize 16777216
```

AMD Ryzen 9 5900X, 32GB DDR4-3200 (16-17-17-35)

| Threads | Speed (GiB/s) |
| ------- | ------------- |
| 1       | ~1.34 GiB/s   |
| 2       | ~2.60 GiB/s   |
| 3       | ~3.88 GiB/s   |
| 4       | ~5.00 GiB/s   |
| 6       | ~6.93 GiB/s   |
| 8       | ~8.62 GiB/s   |
| 12      | ~10.67 GiB/s  |
| 24      | ~9.50 GiB/s ⚠ |

!!! question "Why slower with max thread count?"

    To the best of my knowledge, increased CPU core cache misses.

#### 16M Chunks, Long Mode

```
dotnet ./cli/NexusMods.Archives.Nx.Cli.dll pack --source "textures" --target "out-16M-long.nx" --solidlevel 22 --chunkedlevel 22 --chunksize 16777216
```

AMD Ryzen 9 5900X, 32GB DDR4-3200 (16-17-17-35)

| Threads | Speed (GiB/s) |
| ------- | ------------- |
| 1       | ~1.11 GiB/s   |
| 2       | ~2.08 GiB/s   |
| 3       | ~2.41 GiB/s   |
| 4       | ~2.89 GiB/s   |
| 6       | ~3.70 GiB/s   |
| 8       | ~5.12 GiB/s   |
| 12      | ~5.90 GiB/s   |
| 24      | ~5.33 GiB/s ⚠ |

!!! question "Why so slow here?"

    The data consists of primarily BC7 files, which have multiple internal 'modes', with mode 0
    and 1 generally representing ~70% of the data.

    Because the modes have different structures, this means that in practice, data in a given mode
    will only generally match data from the same mode earlier in the byte stream.

    However, the interleaving of modes, increases the LZ offset distance, and often decreases LZ match length.
    The offset distance increase in particular is harmful to decompression speed, as we're often
    reading data that may not be in any of the CPU caches anymore.

    [dxt-lossless-transform](https://github.com/Sewer56/dxt-lossless-transform) should resolve most
    of this in the near future.

[bench-dataset]: https://u.pcloud.link/publink/show?code=XZf6Tp5Z9CM99OirKGVSSTSQh13MLRBvhiMV