# Dictionary Compression

For reference all code was ran on a 5900X with 2133MHz DDR4 RAM (no OC).
This page just has results of some random tests.

A page of notes/random rambling related to dictionary compression.

## How to run the tests

You can find a test Python scripts in the `tools` folder of the repo, this test script requires that
you have `zstd` available in your `PATH`. On most Linux distributions, you will have this available
out of the box, on Windows you'll need to install it.

Originally I intended these scripts to be throwaways, but kept it for future use.

## Per-file dictionary Size

### Testing on a Random Texture

!!! info "Source: `Whiterun alternative street stone by Pfuscher 2.2-2347-2-2-1578423661.7z`"

    File: `wrstonefloor01.dds`
    Original size: 64.0MiB

#### 110K Dict Size, 128K Blocks

```
Input file size: 64.00 MiB
Target dictionary size: 110.00 KiB (110 KiB (fixed))
Splitting into 1024 KiB blocks...

Overall Size Comparison:
Original size:              132.39 MiB
Dictionary size:            110.00 KiB (0.1%)
Dictionary compressed:      53.90 KiB (0.0%)
Compressed (no dict):       33.99 MiB (25.7%)
Compressed (with dict):     34.34 MiB (25.9%)
Total with dict+blocks:     34.40 MiB (26.0%)

Overall Space Savings:
Without dictionary:         74.32%
With dictionary:            74.02%
Dictionary advantage:       -0.31%

Per-Block Statistics:
Number of blocks: 65
Blocks with ≥4KiB improvement: 0 (0.0%)
Average 4KiB units saved per block: -1.88

Dictionary Advantage (percentage points):
  Min: -2.36%
  Max: -0.14%
  Avg: -0.58%

Compression Ratio:
  Without Dictionary:
    Min: 34.65%
    Max: 50.40%
    Avg: 46.70%
  With Dictionary:
    Min: 32.28%
    Max: 49.64%
    Avg: 46.12%

Most Improved Block:
  block_0045:
    Original: 1.00 MiB
    Without dict: 577.79 KiB (43.58% saved)
    With dict: 579.17 KiB (43.44% saved)
    Advantage: -0.14%
    Bytes saved: -1416.00 B

Least Improved Block:
  block_0064:
    Original: 127.00 B
    Without dict: 83.00 B (34.65% saved)
    With dict: 86.00 B (32.28% saved)
    Advantage: -2.36%
    Bytes saved: -3.00 B
```

Decodes @ 831.4 MB/s with dict.
Decodes @ 1015.8 MB/s without dict.

#### Size/100 Dict Size, 128K Blocks

```
Input file size: 64.00 MiB
Target dictionary size: 655.36 KiB (1/100 of input)
Splitting into 1024 KiB blocks...

Overall Size Comparison:
Original size:              130.63 MiB
Dictionary size:            655.36 KiB (0.5%)
Dictionary compressed:      331.79 KiB (0.2%)
Compressed (no dict):       33.99 MiB (26.0%)
Compressed (with dict):     32.31 MiB (24.7%)
Total with dict+blocks:     32.63 MiB (25.0%)

Overall Space Savings:
Without dictionary:         73.98%
With dictionary:            75.02%
Dictionary advantage:       1.04%

Per-Block Statistics:
Number of blocks: 65
Blocks with ≥4KiB improvement: 64 (98.5%)
Average 4KiB units saved per block: 6.15

Dictionary Advantage (percentage points):
  Min: -7.09%
  Max: 4.59%
  Avg: 2.48%

Compression Ratio:
  Without Dictionary:
    Min: 34.65%
    Max: 50.40%
    Avg: 46.70%
  With Dictionary:
    Min: 27.56%
    Max: 53.22%
    Avg: 49.18%

Most Improved Block:
  block_0045:
    Original: 1.00 MiB
    Without dict: 577.79 KiB (43.58% saved)
    With dict: 530.80 KiB (48.16% saved)
    Advantage: 4.59%
    Bytes saved: 46.98 KiB

Least Improved Block:
  block_0064:
    Original: 127.00 B
    Without dict: 83.00 B (34.65% saved)
    With dict: 92.00 B (27.56% saved)
    Advantage: -7.09%
    Bytes saved: -9.00 B
```

Decodes @ 744.4 MB/s with dict.
Decodes @ 1015.8 MB/s without dict.

Speed tanks, presumably due to failed branch prediction more often, either that or
L2 cache size. I haven't dug into instruction by instruction to find out.

#### Size/100 Dict Size, 1M Blocks

```
Input file size: 64.00 MiB
Target dictionary size: 655.36 KiB (1/100 of input)
Splitting into 1024 KiB blocks...

Overall Size Comparison:
Original size:              130.63 MiB
Dictionary size:            655.36 KiB (0.5%)
Dictionary compressed:      331.79 KiB (0.2%)
Compressed (no dict):       33.99 MiB (26.0%)
Compressed (with dict):     32.31 MiB (24.7%)
Total with dict+blocks:     32.63 MiB (25.0%)

Overall Space Savings:
Without dictionary:         73.98%
With dictionary:            75.02%
Dictionary advantage:       1.04%

Per-Block Statistics:
Number of blocks: 65
Blocks with ≥4KiB improvement: 64 (98.5%)
Average 4KiB units saved per block: 6.15

Dictionary Advantage (percentage points):
  Min: -7.09%
  Max: 4.59%
  Avg: 2.48%

Compression Ratio:
  Without Dictionary:
    Min: 34.65%
    Max: 50.40%
    Avg: 46.70%
  With Dictionary:
    Min: 27.56%
    Max: 53.22%
    Avg: 49.18%

Most Improved Block:
  block_0045:
    Original: 1.00 MiB
    Without dict: 577.79 KiB (43.58% saved)
    With dict: 530.80 KiB (48.16% saved)
    Advantage: 4.59%
    Bytes saved: 46.98 KiB

Least Improved Block:
  block_0064:
    Original: 127.00 B
    Without dict: 83.00 B (34.65% saved)
    With dict: 92.00 B (27.56% saved)
    Advantage: -7.09%
    Bytes saved: -9.00 B
```

Decodes @ 744.4 MB/s with dict.
Decodes @ 1015.8 MB/s without dict.

## Per-file dictionary Speed

!!! info "This section contains only raw data"

### 1M Blocks, 256K Dict Size

```bash
/home/sewer/Project/sewer56-archives-nx/tools/benchmark-single-file-dict.py  /home/sewer/Downloads/202x/textures/architecture/solitude --block-size 1024 --dict-size 262144 -e dds
```

??? info "Per file stats"

    ```
    scastlecol01.dds:
    Average speed without dict: 1078.49 MB/s
    Average speed with dict: 1302.66 MB/s
    Average difference: +224.17 MB/s (+20.8%)

    sclovers01.dds:
    Average speed without dict: 1047.38 MB/s
    Average speed with dict: 783.59 MB/s
    Average difference: -263.80 MB/s (-25.2%)

    sdetails01.dds:
    Average speed without dict: 789.62 MB/s
    Average speed with dict: 629.51 MB/s
    Average difference: -160.12 MB/s (-20.3%)

    sdetint01.dds:
    Average speed without dict: 826.24 MB/s
    Average speed with dict: 710.49 MB/s
    Average difference: -115.75 MB/s (-14.0%)

    sdirt01.dds:
    Average speed without dict: 888.40 MB/s
    Average speed with dict: 953.77 MB/s
    Average difference: +65.38 MB/s (+7.4%)

    sdirt02.dds:
    Average speed without dict: 1127.68 MB/s
    Average speed with dict: 1148.15 MB/s
    Average difference: +20.46 MB/s (+1.8%)

    sdoor02.dds:
    Average speed without dict: 925.98 MB/s
    Average speed with dict: 800.76 MB/s
    Average difference: -125.23 MB/s (-13.5%)

    sdoor03.dds:
    Average speed without dict: 892.41 MB/s
    Average speed with dict: 907.79 MB/s
    Average difference: +15.38 MB/s (+1.7%)

    sdragonhead01.dds:
    Average speed without dict: 866.21 MB/s
    Average speed with dict: 1045.39 MB/s
    Average difference: +179.18 MB/s (+20.7%)

    sdragontile01.dds:
    Average speed without dict: 839.15 MB/s
    Average speed with dict: 685.96 MB/s
    Average difference: -153.19 MB/s (-18.3%)

    sfloorhouse01.dds:
    Average speed without dict: 1048.65 MB/s
    Average speed with dict: 886.49 MB/s
    Average difference: -162.16 MB/s (-15.5%)

    sfloorhouse02.dds:
    Average speed without dict: 975.19 MB/s
    Average speed with dict: 891.84 MB/s
    Average difference: -83.35 MB/s (-8.5%)

    sgrass01.dds:
    Average speed without dict: 939.30 MB/s
    Average speed with dict: 1128.20 MB/s
    Average difference: +188.90 MB/s (+20.1%)

    sintcoloumn02.dds:
    Average speed without dict: 840.77 MB/s
    Average speed with dict: 685.05 MB/s
    Average difference: -155.72 MB/s (-18.5%)

    sintfloor01.dds:
    Average speed without dict: 786.31 MB/s
    Average speed with dict: 573.08 MB/s
    Average difference: -213.23 MB/s (-27.1%)

    smill01.dds:
    Average speed without dict: 858.45 MB/s
    Average speed with dict: 971.74 MB/s
    Average difference: +113.29 MB/s (+13.2%)

    smoss01.dds:
    Average speed without dict: 1623.20 MB/s
    Average speed with dict: 1191.50 MB/s
    Average difference: -431.70 MB/s (-26.6%)

    smoss01_m.dds:
    Average speed without dict: 2337.08 MB/s
    Average speed with dict: 1937.47 MB/s
    Average difference: -399.61 MB/s (-17.1%)

    smoss02walls.dds:
    Average speed without dict: 2698.35 MB/s
    Average speed with dict: 2506.42 MB/s
    Average difference: -191.92 MB/s (-7.1%)

    sroofslate01.dds:
    Average speed without dict: 827.25 MB/s
    Average speed with dict: 857.58 MB/s
    Average difference: +30.33 MB/s (+3.7%)

    sslatebaseint01.dds:
    Average speed without dict: 818.95 MB/s
    Average speed with dict: 599.92 MB/s
    Average difference: -219.03 MB/s (-26.7%)

    ssteps01.dds:
    Average speed without dict: 1219.74 MB/s
    Average speed with dict: 1328.23 MB/s
    Average difference: +108.50 MB/s (+8.9%)

    ssteps02.dds:
    Average speed without dict: 1176.62 MB/s
    Average speed with dict: 1194.79 MB/s
    Average difference: +18.17 MB/s (+1.5%)

    sstonebase01.dds:
    Average speed without dict: 836.94 MB/s
    Average speed with dict: 765.79 MB/s
    Average difference: -71.14 MB/s (-8.5%)

    sstonefloor01.dds:
    Average speed without dict: 828.64 MB/s
    Average speed with dict: 666.23 MB/s
    Average difference: -162.41 MB/s (-19.6%)

    sstonefloortrim01.dds:
    Average speed without dict: 879.63 MB/s
    Average speed with dict: 884.46 MB/s
    Average difference: +4.83 MB/s (+0.5%)

    sstonestep01.dds:
    Average speed without dict: 847.76 MB/s
    Average speed with dict: 705.67 MB/s
    Average difference: -142.09 MB/s (-16.8%)

    sstonewall.dds:
    Average speed without dict: 983.41 MB/s
    Average speed with dict: 823.37 MB/s
    Average difference: -160.03 MB/s (-16.3%)

    sstonewall02.dds:
    Average speed without dict: 789.47 MB/s
    Average speed with dict: 572.64 MB/s
    Average difference: -216.83 MB/s (-27.5%)

    sstonewall03.dds:
    Average speed without dict: 799.66 MB/s
    Average speed with dict: 636.41 MB/s
    Average difference: -163.26 MB/s (-20.4%)

    sstuccowall.dds:
    Average speed without dict: 918.42 MB/s
    Average speed with dict: 646.83 MB/s
    Average difference: -271.59 MB/s (-29.6%)

    sstuccowall02.dds:
    Average speed without dict: 1150.37 MB/s
    Average speed with dict: 1003.71 MB/s
    Average difference: -146.66 MB/s (-12.7%)

    sstuccowallint01.dds:
    Average speed without dict: 995.63 MB/s
    Average speed with dict: 852.71 MB/s
    Average difference: -142.92 MB/s (-14.4%)

    strims01.dds:
    Average speed without dict: 1110.67 MB/s
    Average speed with dict: 1021.41 MB/s
    Average difference: -89.26 MB/s (-8.0%)

    swoodbeam01.dds:
    Average speed without dict: 797.30 MB/s
    Average speed with dict: 579.73 MB/s
    Average difference: -217.57 MB/s (-27.3%)

    swoodbeam02.dds:
    Average speed without dict: 804.17 MB/s
    Average speed with dict: 665.80 MB/s
    Average difference: -138.37 MB/s (-17.2%)

    swoodcolumn01.dds:
    Average speed without dict: 1343.45 MB/s
    Average speed with dict: 1118.17 MB/s
    Average difference: -225.28 MB/s (-16.8%)

    swooddet01.dds:
    Average speed without dict: 1219.21 MB/s
    Average speed with dict: 1178.85 MB/s
    Average difference: -40.37 MB/s (-3.3%)

    swoodfloor01.dds:
    Average speed without dict: 780.45 MB/s
    Average speed with dict: 624.26 MB/s
    Average difference: -156.19 MB/s (-20.0%)

    swoodplanks01.dds:
    Average speed without dict: 837.44 MB/s
    Average speed with dict: 659.61 MB/s
    Average difference: -177.83 MB/s (-21.2%)

    swoodplaster01.dds:
    Average speed without dict: 872.33 MB/s
    Average speed with dict: 962.22 MB/s
    Average difference: +89.89 MB/s (+10.3%)

    swoodstep01.dds:
    Average speed without dict: 831.74 MB/s
    Average speed with dict: 718.43 MB/s
    Average difference: -113.31 MB/s (-13.6%)
    ```

```
Overall averages:
  Without dictionary: 991.89 MB/s
  With dictionary: 849.61 MB/s
  Difference: -142.27 MB/s (-14.3%)
```

### 1M Blocks, FileSize/100 Dict Size

??? info "Per file stats"

    ```
    scastlecol01.dds:
    Average speed without dict: 1237.72 MB/s
    Average speed with dict: 1024.67 MB/s
    Average difference: -213.05 MB/s (-17.2%)

    sclovers01.dds:
    Average speed without dict: 1055.95 MB/s
    Average speed with dict: 559.51 MB/s
    Average difference: -496.43 MB/s (-47.0%)

    sdetails01.dds:
    Average speed without dict: 804.91 MB/s
    Average speed with dict: 621.43 MB/s
    Average difference: -183.48 MB/s (-22.8%)

    sdetint01.dds:
    Average speed without dict: 837.64 MB/s
    Average speed with dict: 660.35 MB/s
    Average difference: -177.30 MB/s (-21.2%)

    sdirt01.dds:
    Average speed without dict: 882.35 MB/s
    Average speed with dict: 711.81 MB/s
    Average difference: -170.54 MB/s (-19.3%)

    sdirt02.dds:
    Average speed without dict: 1135.32 MB/s
    Average speed with dict: 884.66 MB/s
    Average difference: -250.67 MB/s (-22.1%)

    sdoor02.dds:
    Average speed without dict: 929.72 MB/s
    Average speed with dict: 780.60 MB/s
    Average difference: -149.12 MB/s (-16.0%)

    sdoor03.dds:
    Average speed without dict: 907.18 MB/s
    Average speed with dict: 902.80 MB/s
    Average difference: -4.38 MB/s (-0.5%)

    sdragonhead01.dds:
    Average speed without dict: 879.56 MB/s
    Average speed with dict: 811.30 MB/s
    Average difference: -68.27 MB/s (-7.8%)

    sdragontile01.dds:
    Average speed without dict: 852.54 MB/s
    Average speed with dict: 714.80 MB/s
    Average difference: -137.74 MB/s (-16.2%)

    sfloorhouse01.dds:
    Average speed without dict: 1047.07 MB/s
    Average speed with dict: 681.13 MB/s
    Average difference: -365.95 MB/s (-34.9%)

    sfloorhouse02.dds:
    Average speed without dict: 964.13 MB/s
    Average speed with dict: 1141.60 MB/s
    Average difference: +177.47 MB/s (+18.4%)

    sgrass01.dds:
    Average speed without dict: 942.77 MB/s
    Average speed with dict: 764.08 MB/s
    Average difference: -178.68 MB/s (-19.0%)

    sintcoloumn02.dds:
    Average speed without dict: 832.30 MB/s
    Average speed with dict: 694.34 MB/s
    Average difference: -137.96 MB/s (-16.6%)

    sintfloor01.dds:
    Average speed without dict: 775.15 MB/s
    Average speed with dict: 528.06 MB/s
    Average difference: -247.09 MB/s (-31.9%)

    smill01.dds:
    Average speed without dict: 846.54 MB/s
    Average speed with dict: 722.30 MB/s
    Average difference: -124.24 MB/s (-14.7%)

    smoss01.dds:
    Average speed without dict: 1604.23 MB/s
    Average speed with dict: 1662.51 MB/s
    Average difference: +58.27 MB/s (+3.6%)

    smoss01_m.dds:
    Average speed without dict: 2354.87 MB/s
    Average speed with dict: 2135.83 MB/s
    Average difference: -219.05 MB/s (-9.3%)

    smoss02walls.dds:
    Average speed without dict: 2713.32 MB/s
    Average speed with dict: 2573.90 MB/s
    Average difference: -139.42 MB/s (-5.1%)

    sroofslate01.dds:
    Average speed without dict: 882.21 MB/s
    Average speed with dict: 828.31 MB/s
    Average difference: -53.90 MB/s (-6.1%)

    sslatebaseint01.dds:
    Average speed without dict: 827.65 MB/s
    Average speed with dict: 560.33 MB/s
    Average difference: -267.32 MB/s (-32.3%)

    ssteps01.dds:
    Average speed without dict: 1265.91 MB/s
    Average speed with dict: 1052.24 MB/s
    Average difference: -213.67 MB/s (-16.9%)

    ssteps02.dds:
    Average speed without dict: 1186.68 MB/s
    Average speed with dict: 1008.74 MB/s
    Average difference: -177.94 MB/s (-15.0%)

    sstonebase01.dds:
    Average speed without dict: 855.47 MB/s
    Average speed with dict: 712.75 MB/s
    Average difference: -142.73 MB/s (-16.7%)

    sstonefloor01.dds:
    Average speed without dict: 818.08 MB/s
    Average speed with dict: 592.09 MB/s
    Average difference: -226.00 MB/s (-27.6%)

    sstonefloortrim01.dds:
    Average speed without dict: 862.94 MB/s
    Average speed with dict: 719.99 MB/s
    Average difference: -142.95 MB/s (-16.6%)

    sstonestep01.dds:
    Average speed without dict: 828.01 MB/s
    Average speed with dict: 639.10 MB/s
    Average difference: -188.90 MB/s (-22.8%)

    sstonewall.dds:
    Average speed without dict: 992.14 MB/s
    Average speed with dict: 792.04 MB/s
    Average difference: -200.10 MB/s (-20.2%)

    sstonewall02.dds:
    Average speed without dict: 799.21 MB/s
    Average speed with dict: 531.20 MB/s
    Average difference: -268.01 MB/s (-33.5%)

    sstonewall03.dds:
    Average speed without dict: 808.39 MB/s
    Average speed with dict: 569.15 MB/s
    Average difference: -239.24 MB/s (-29.6%)

    sstuccowall.dds:
    Average speed without dict: 904.69 MB/s
    Average speed with dict: 741.90 MB/s
    Average difference: -162.79 MB/s (-18.0%)

    sstuccowall02.dds:
    Average speed without dict: 1150.85 MB/s
    Average speed with dict: 831.19 MB/s
    Average difference: -319.66 MB/s (-27.8%)

    sstuccowallint01.dds:
    Average speed without dict: 1016.91 MB/s
    Average speed with dict: 692.57 MB/s
    Average difference: -324.34 MB/s (-31.9%)

    strims01.dds:
    Average speed without dict: 1098.78 MB/s
    Average speed with dict: 840.70 MB/s
    Average difference: -258.08 MB/s (-23.5%)

    swoodbeam01.dds:
    Average speed without dict: 792.29 MB/s
    Average speed with dict: 532.20 MB/s
    Average difference: -260.09 MB/s (-32.8%)

    swoodbeam02.dds:
    Average speed without dict: 793.53 MB/s
    Average speed with dict: 607.36 MB/s
    Average difference: -186.17 MB/s (-23.5%)

    swoodcolumn01.dds:
    Average speed without dict: 1374.12 MB/s
    Average speed with dict: 1098.21 MB/s
    Average difference: -275.90 MB/s (-20.1%)

    swooddet01.dds:
    Average speed without dict: 1218.22 MB/s
    Average speed with dict: 1159.94 MB/s
    Average difference: -58.28 MB/s (-4.8%)

    swoodfloor01.dds:
    Average speed without dict: 807.24 MB/s
    Average speed with dict: 692.47 MB/s
    Average difference: -114.77 MB/s (-14.2%)

    swoodplanks01.dds:
    Average speed without dict: 835.01 MB/s
    Average speed with dict: 674.77 MB/s
    Average difference: -160.23 MB/s (-19.2%)

    swoodplaster01.dds:
    Average speed without dict: 883.10 MB/s
    Average speed with dict: 684.30 MB/s
    Average difference: -198.80 MB/s (-22.5%)

    swoodstep01.dds:
    Average speed without dict: 841.06 MB/s
    Average speed with dict: 703.49 MB/s
    Average difference: -137.57 MB/s (-16.4%)
    ```

```
Overall averages:
  Without dictionary: 996.19 MB/s
  With dictionary: 779.49 MB/s
  Difference: -216.70 MB/s (-21.8%)
```

#### 110KB Dict Size Limit, 1M Blocks

```
Overall averages:
  Without dictionary: 993.59 MB/s
  With dictionary: 896.59 MB/s
  Difference: -97.00 MB/s (-9.8%)
```

#### 110KB Dict Size Limit, 64K Blocks

```
Overall averages:
  Without dictionary: 995.76 MB/s
  With dictionary: 896.12 MB/s
  Difference: -99.64 MB/s (-10.0%)
```

### Per Extension Stats

!!! info "All done at zstd level 12"

110KB dict size limit, unless stated otherwise.

#### C++ Code

Tested on `cblib` (Charles Bloom).

C++ Code, 64KB blocks, 64KB file size limit

```
Looking for .cpp files under 64.00 KiB...
Target block size: 64.00 KiB
Found 128 files
Arranged into 22 blocks

=== Compression Summary ===
Total original size: 1.11 MiB

Compressed sizes:
  Individual files (no dict):   303.48 KiB (3.74x)
  Individual files (with dict):  222.36 KiB (5.10x)
  Solid blocks (no dict):        277.92 KiB (4.08x)
  Solid blocks (with dict):      216.52 KiB (5.24x)

Space savings vs individual (no dict):
  Dictionary advantage:         81.12 KiB (26.7%)
  Solid block advantage:        25.56 KiB (8.4%)
  Solid block + dict advantage: 86.96 KiB (28.7%)

Average decompression speeds:
  Individual files (no dict):   1170.76 MB/s
  Individual files (with dict):  1413.42 MB/s
  Solid blocks (no dict):        1319.67 MB/s
```

128KB blocks, 64KB file size limit

```
Looking for .cpp files under 64.00 KiB...
Target block size: 128.00 KiB
Found 128 files
Arranged into 10 blocks

=== Compression Summary ===
Total original size: 1.11 MiB

Compressed sizes:
  Individual files (no dict):   303.48 KiB (3.74x)
  Individual files (with dict):  222.36 KiB (5.10x)
  Solid blocks (no dict):        268.30 KiB (4.23x)
  Solid blocks (with dict):      215.92 KiB (5.25x)

Space savings vs individual (no dict):
  Dictionary advantage:         81.12 KiB (26.7%)
  Solid block advantage:        35.18 KiB (11.6%)
  Solid block + dict advantage: 87.56 KiB (28.9%)

Average decompression speeds:
  Individual files (no dict):   1180.18 MB/s
  Individual files (with dict):  1484.81 MB/s
  Solid blocks (no dict):        1325.63 MB/s
  Solid blocks (with dict):      1474.08 MB/s
```

#### DDS Textures

!!! info "Source: `Interesting NPCs 3DNPC SE - Loose-29194-4-3-6-1582211680.7z`"

128KB blocks, 64KB file size limit

```
=== Compression Summary ===
Total original size: 984.96 KiB

Compressed sizes:
  Individual files (no dict):   731.52 KiB (1.35x)
  Individual files (with dict):  658.45 KiB (1.50x)
  Solid blocks (no dict):        728.13 KiB (1.35x)
  Solid blocks (with dict):      658.19 KiB (1.50x)

Space savings vs individual (no dict):
  Dictionary advantage:         73.07 KiB (10.0%)
  Solid block advantage:        3.39 KiB (0.5%)
  Solid block + dict advantage: 73.33 KiB (10.0%)

Average decompression speeds:
  Individual files (no dict):   784.33 MB/s
  Individual files (with dict):  812.99 MB/s
```

128KB blocks, 128KB file size limit

```
=== Compression Summary ===
Total original size: 4.89 MiB

Compressed sizes:
  Individual files (no dict):   3.53 MiB (1.38x)
  Individual files (with dict):  3.47 MiB (1.41x)
  Solid blocks (no dict):        3.53 MiB (1.39x)
  Solid blocks (with dict):      3.47 MiB (1.41x)

Space savings vs individual (no dict):
  Dictionary advantage:         61.98 KiB (1.7%)
  Solid block advantage:        3.39 KiB (0.1%)
  Solid block + dict advantage: 62.08 KiB (1.7%)

Average decompression speeds:
  Individual files (no dict):   740.73 MB/s
  Individual files (with dict):  724.82 MB/s
  Solid blocks (no dict):        689.80 MB/s
  Solid blocks (with dict):      693.06 MB/s
```

Unlike big files, smaller files are not hurt by use of a dictionary.

#### Skyrim Binary Scripts (.pex)

!!! info "Source: `Interesting NPCs 3DNPC SE - Loose-29194-4-3-6-1582211680.7z`"

128KB blocks, 64KB file size limit

```
/home/sewer/Project/sewer56-archives-nx/tools/benchmark-dict-over-extension.py /home/sewer/Downloads/3dnpc -e pex --block-size 131072
Looking for .pex files under 64.00 KiB...
Target block size: 128.00 KiB
Found 6904 files
Arranged into 45 blocks

=== Compression Summary ===
Total original size: 5.53 MiB

Compressed sizes:
  Individual files (no dict):   3.44 MiB (1.61x)
  Individual files (with dict):  685.92 KiB (8.26x)
  Solid blocks (no dict):        444.64 KiB (12.74x)
  Solid blocks (with dict):      366.99 KiB (15.44x)

Space savings vs individual (no dict):
  Dictionary advantage:         2.77 MiB (80.5%)
  Solid block advantage:        3.00 MiB (87.4%)
  Solid block + dict advantage: 3.08 MiB (89.6%)

Average decompression speeds:
  Individual files (no dict):   367.11 MB/s
  Individual files (with dict):  2142.75 MB/s
  Solid blocks (no dict):        4184.56 MB/s
  Solid blocks (with dict):      4569.94 MB/s
```

This one was too interesting not to post.

#### Skyrim Voices (.fuz)

128KB blocks, 64KB file size limit

```
/home/sewer/Project/sewer56-archives-nx/tools/benchmark-dict-over-extension.py /home/sewer/Downloads/3dnpc/sound/voice/3dnpc.esp/zorafairchildvoice -e fuz --block-size 131072

=== Compression Summary ===
Total original size: 42.21 MiB

Compressed sizes:
  Individual files (no dict):   36.61 MiB (1.15x)
  Individual files (with dict):  36.20 MiB (1.17x)
  Solid blocks (no dict):        36.29 MiB (1.16x)
  Solid blocks (with dict):      36.03 MiB (1.17x)

Space savings vs individual (no dict):
  Dictionary advantage:         422.70 KiB (1.1%)
  Solid block advantage:        327.61 KiB (0.9%)
  Solid block + dict advantage: 598.52 KiB (1.6%)

Average decompression speeds:
  Individual files (no dict):   2110.25 MB/s
  Individual files (with dict):  1790.29 MB/s
  Solid blocks (no dict):        1843.08 MB/s
  Solid blocks (with dict):      1955.87 MB/s
```

## Takeaways

!!! info "From the tests here, and many more."

- Dictionary compression is effective for files <128KiB.
- Dictionary compression over the file itself is generally ineffective.
- Train on only start of files was attempted, in the hopes that rest of file yields constant branch
  prediction hits as the dictionary wouldn't be used. That didn't quite work out.
- Dictionary on only first blocks of large files yielded negligible overall difference.

When archiving unknown data,