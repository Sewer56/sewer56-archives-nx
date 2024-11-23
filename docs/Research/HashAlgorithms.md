# Hash Algorithm Benchmarks

Tested on AMD Ryzen 9 5900X, 32GB DDR4-3000 (16-17-17-35)
[Tested on the Rust rapidhash port test suite][rapidhash-testsuite]

## Pure Hashing Speed

!!! note "Non-portable implementations are marked with an asterisk (*)."

!!! note "This measures the performance of the streaming API only."

    This doesn't measure the performance of the `oneshot` implementation; i.e.
    when the full length of data is known. Only the rust `Hasher` streaming API
    is measured; this can be very inefficient for some implementations.

    The `oneshot` implementation of XXH3 sits between `rapidhash` and `wyhash` for
    reference, before starting to run away at 512 bytes and beyond due to hardware
    acceleration.

### Short String Performance (2-64 characters)

| Implementation | str_2        | str_8        | str_16       | str_64       |
| -------------- | ------------ | ------------ | ------------ | ------------ |
| fxhash         | 1.57ns       | 1.36ns       | 1.77ns       | 3.83ns       |
| gxhash*        | 2.09ns       | 2.08ns       | 2.08ns       | 2.54ns       |
| rustc-hash     | 2.24ns       | 2.03ns       | 1.99ns       | 3.46ns       |
| rapidhash_raw  | 2.67ns       | 2.47ns       | 2.47ns       | 4.08ns       |
| rapidhash      | 2.75ns       | 2.44ns       | 2.49ns       | 4.09ns       |
| wyhash_raw     | 3.15ns       | 3.19ns       | 3.29ns       | 4.06ns       |
| ahash          | 3.59ns       | 3.59ns       | 3.82ns       | 5.55ns       |
| wyhash         | 3.60ns       | 3.58ns       | 3.81ns       | 4.78ns       |
| metrohash*     | 4.02ns       | 3.61ns       | 4.49ns       | 8.01ns       |
| t1ha*          | 4.70ns       | 4.67ns       | 4.47ns       | 7.87ns       |
| default        | 5.56ns       | 6.34ns       | 7.37ns       | 13.81ns      |
| xxhash64       | 8.71ns       | 8.02ns       | 8.63ns       | 12.55ns      |
| farmhash*      | 12.24ns      | 15.10ns      | 15.40ns      | 34.21ns      |
| seahash        | 13.23ns      | 8.91ns       | 9.65ns       | 13.69ns      |
| xxhash3_64     | 21.88ns      | 21.78ns      | 21.53ns      | 23.28ns      |
| highwayhash    | 33.98ns      | 32.63ns      | 29.31ns      | 31.60ns      |

### Long String Performance (100+ characters)

| Implementation | str_100      | str_177      | str_256      | str_1024      | str_4096      |
| -------------- | ------------ | ------------ | ------------ | ------------- | ------------- |
| rapidhash_raw  | 4.38ns       | 7.65ns       | 9.05ns       | 31.50ns       | 128.07ns      |
| gxhash*        | 4.36ns       | 5.31ns       | 5.90ns       | 16.71ns       | 54.33ns       |
| rapidhash      | 4.48ns       | 7.16ns       | 8.44ns       | 31.57ns       | 130.17ns      |
| rustc-hash     | 4.67ns       | 7.42ns       | 9.62ns       | 39.64ns       | 175.03ns      |
| fxhash         | 6.81ns       | 14.62ns      | 21.00ns      | 123.32ns      | 549.44ns      |
| wyhash_raw     | 7.00ns       | 10.19ns      | 12.96ns      | 47.94ns       | 193.76ns      |
| wyhash         | 7.34ns       | 11.47ns      | 13.90ns      | 52.58ns       | 204.68ns      |
| t1ha*          | 9.50ns       | 12.20ns      | 15.92ns      | 51.76ns       | 206.03ns      |
| metrohash*     | 10.87ns      | 16.75ns      | 18.10ns      | 54.97ns       | 202.29ns      |
| ahash          | 12.71ns      | 16.25ns      | 20.44ns      | 87.60ns       | 357.70ns      |
| xxhash64       | 14.31ns      | 19.91ns      | 22.28ns      | 66.61ns       | 235.66ns      |
| seahash        | 16.92ns      | 25.39ns      | 31.93ns      | 114.53ns      | 427.51ns      |
| default        | 18.54ns      | 31.04ns      | 40.10ns      | 150.11ns      | 577.65ns      |
| xxhash3_64     | 27.80ns      | 33.27ns      | 28.49ns      | 48.89ns       | 96.06ns       |
| highwayhash    | 39.86ns      | 44.03ns      | 41.78ns      | 84.37ns       | 251.55ns      |
| farmhash*      | 45.33ns      | 58.66ns      | 61.79ns      | 103.12ns      | 275.50ns      |

### Miscellaneous Types (u64 and object)

| Implementation | u64          | object        |
| -------------- | ------------ | ------------- |
| fxhash         | 281ps        | 7.43ns        |
| rustc-hash     | 404ps        | 4.36ns        |
| gxhash*        | 539ps        | 6.31ns        |
| wyhash         | 819ps        | 14.40ns       |
| rapidhash      | 999ps        | 10.90ns       |
| metrohash*     | 1.06ns       | 37.65ns       |
| wyhash_raw     | 3.40ns       | -             |
| seahash        | 8.53ns       | 68.82ns       |
| xxhash64       | 8.58ns       | 40.99ns       |
| default        | 8.71ns       | 35.28ns       |
| farmhash*      | 10.09ns      | 101.06ns      |
| highwayhash    | 34.85ns      | 52.22ns       |
| xxhash3_64     | 21.24ns      | 46.27ns       |

### Observations

Key observations:

1. gxhash dominates long data but isn't portable
2. For short strings, fxhash and rustc-hash are very competitive speed wise
3. rapidhash variants show very consistent performance
4. For u64/object hashing, rustc-hash and fxhash are the clear winners
5. Rust implementation of `xxhash` has high startup overhead. But wins out on long data due to SSE2/AVX2.

## Hashmaps

!!! note "Non-portable implementations are marked with an asterisk (*)."

### Text-Based Map Performance

| Implementation   | 1000_small   | 10000_emails  | 450000_words | Points |
| ---------------- | ------------ | ------------- | ------------ | ------ |
| fxhash           | 1 (32.53 µs) | 3 (510.30 µs) | 1 (63.33 ms) | 5      |
| rapidhash        | 3 (41.78 µs) | 1 (466.93 µs) | 2 (63.98 ms) | 6      |
| rapidhash_inline | 4 (45.61 µs) | 2 (476.56 µs) | 3 (67.04 ms) | 9      |
| gxhash*          | 5 (48.96 µs) | 4 (552.04 µs) | 4 (67.68 ms) | 13     |
| wyhash           | 2 (40.77 µs) | 5 (594.52 µs) | 5 (71.94 ms) | 12     |
| default          | 6 (54.69 µs) | 6 (778.47 µs) | 6 (90.93 ms) | 18     |

### Non-Text Map Performance (Numbers and Structs)

| Implementation   | 100000_u64  | 10000_struct | Points |
| ---------------- | ----------- | ------------ | ------ |
| rapidhash_inline | 2 (1.53 ms) | 3 (1.76 ms)  | 5      |
| rapidhash        | 3 (1.56 ms) | 2 (1.73 ms)  | 5      |
| gxhash*          | 5 (1.88 ms) | 1 (1.67 ms)  | 6      |
| fxhash           | 1 (1.45 ms) | 5 (2.41 ms)  | 6      |
| wyhash           | 4 (1.59 ms) | 4 (2.09 ms)  | 8      |
| default          | 6 (3.00 ms) | 6 (3.81 ms)  | 12     |

### Observations

1. For text:
   - fxhash and rapidhash are clearly the leaders
   - rapidhash variants show very consistent performance
   - default hasher significantly trails all others

2. For non-text:
   - rapidhash variants show excellent all-round performance
   - gxhash excels at struct handling but isn't portable
   - fxhash is best for integers but struggles with structs
   - default hasher is consistently slowest

## Raw Output

```
hash/rapidhash/str_2    time:   [2.7374 ns 2.7505 ns 2.7663 ns]
                        thrpt:  [689.50 MiB/s 693.46 MiB/s 696.77 MiB/s]
hash/rapidhash/str_8    time:   [2.4324 ns 2.4443 ns 2.4568 ns]
                        thrpt:  [3.0327 GiB/s 3.0482 GiB/s 3.0631 GiB/s]
hash/rapidhash/str_16   time:   [2.4820 ns 2.4866 ns 2.4917 ns]
                        thrpt:  [5.9804 GiB/s 5.9925 GiB/s 6.0037 GiB/s]
hash/rapidhash/str_64   time:   [4.0835 ns 4.0903 ns 4.0977 ns]
                        thrpt:  [14.546 GiB/s 14.572 GiB/s 14.596 GiB/s]
hash/rapidhash/str_100  time:   [4.4564 ns 4.4783 ns 4.5038 ns]
                        thrpt:  [20.678 GiB/s 20.796 GiB/s 20.898 GiB/s]
hash/rapidhash/str_177  time:   [7.1319 ns 7.1571 ns 7.1860 ns]
                        thrpt:  [22.940 GiB/s 23.032 GiB/s 23.114 GiB/s]
hash/rapidhash/str_256  time:   [8.4066 ns 8.4361 ns 8.4736 ns]
                        thrpt:  [28.137 GiB/s 28.262 GiB/s 28.361 GiB/s]
hash/rapidhash/str_1024 time:   [31.451 ns 31.573 ns 31.709 ns]
                        thrpt:  [30.076 GiB/s 30.205 GiB/s 30.322 GiB/s]
hash/rapidhash/str_4096 time:   [129.88 ns 130.17 ns 130.49 ns]
                        thrpt:  [29.234 GiB/s 29.305 GiB/s 29.371 GiB/s]
hash/rapidhash/u8       time:   [814.73 ps 817.15 ps 819.83 ps]
                        thrpt:  [1.2198 Gelem/s 1.2238 Gelem/s 1.2274 Gelem/s]
hash/rapidhash/u16      time:   [1.4286 ns 1.4363 ns 1.4432 ns]
                        thrpt:  [692.88 Melem/s 696.24 Melem/s 699.98 Melem/s]
hash/rapidhash/u32      time:   [1.0664 ns 1.0692 ns 1.0720 ns]
                        thrpt:  [932.88 Melem/s 935.30 Melem/s 937.76 Melem/s]
hash/rapidhash/u64      time:   [994.42 ps 998.99 ps 1.0045 ns]
                        thrpt:  [995.51 Melem/s 1.0010 Gelem/s 1.0056 Gelem/s]
hash/rapidhash/u128     time:   [1.1344 ns 1.1405 ns 1.1469 ns]
                        thrpt:  [871.95 Melem/s 876.78 Melem/s 881.55 Melem/s]
hash/rapidhash/object   time:   [10.881 ns 10.899 ns 10.921 ns]
                        thrpt:  [91.570 Melem/s 91.749 Melem/s 91.907 Melem/s]
hash/rapidhash/object_inline
                        time:   [7.7032 ns 7.7119 ns 7.7222 ns]
                        thrpt:  [129.50 Melem/s 129.67 Melem/s 129.82 Melem/s]

hash/rapidhash_raw/str_2
                        time:   [2.6562 ns 2.6685 ns 2.6825 ns]
                        thrpt:  [711.04 MiB/s 714.76 MiB/s 718.08 MiB/s]
hash/rapidhash_raw/str_8
                        time:   [2.4625 ns 2.4712 ns 2.4808 ns]
                        thrpt:  [3.0033 GiB/s 3.0150 GiB/s 3.0256 GiB/s]
hash/rapidhash_raw/str_16
                        time:   [2.4591 ns 2.4652 ns 2.4720 ns]
                        thrpt:  [6.0281 GiB/s 6.0445 GiB/s 6.0596 GiB/s]
hash/rapidhash_raw/str_64
                        time:   [4.0552 ns 4.0800 ns 4.1134 ns]
                        thrpt:  [14.490 GiB/s 14.609 GiB/s 14.698 GiB/s]
hash/rapidhash_raw/str_100
                        time:   [4.3596 ns 4.3751 ns 4.3930 ns]
                        thrpt:  [21.200 GiB/s 21.287 GiB/s 21.363 GiB/s]
hash/rapidhash_raw/str_177
                        time:   [7.6111 ns 7.6546 ns 7.7097 ns]
                        thrpt:  [21.381 GiB/s 21.535 GiB/s 21.658 GiB/s]
hash/rapidhash_raw/str_256
                        time:   [8.9768 ns 9.0473 ns 9.1406 ns]
                        thrpt:  [26.084 GiB/s 26.352 GiB/s 26.560 GiB/s]
hash/rapidhash_raw/str_1024
                        time:   [31.332 ns 31.502 ns 31.708 ns]
                        thrpt:  [30.076 GiB/s 30.273 GiB/s 30.438 GiB/s]
hash/rapidhash_raw/str_4096
                        time:   [127.39 ns 128.07 ns 128.90 ns]
                        thrpt:  [29.595 GiB/s 29.786 GiB/s 29.945 GiB/s]
hash/rapidhash_raw/u64  time:   [1.0009 ns 1.0041 ns 1.0078 ns]
                        thrpt:  [992.25 Melem/s 995.91 Melem/s 999.06 Melem/s]

hash/default/str_2      time:   [5.5402 ns 5.5628 ns 5.5861 ns]
                        thrpt:  [341.44 MiB/s 342.88 MiB/s 344.28 MiB/s]
hash/default/str_8      time:   [6.3260 ns 6.3388 ns 6.3531 ns]
                        thrpt:  [1.1727 GiB/s 1.1754 GiB/s 1.1778 GiB/s]
hash/default/str_16     time:   [7.3523 ns 7.3684 ns 7.3890 ns]
                        thrpt:  [2.0167 GiB/s 2.0223 GiB/s 2.0267 GiB/s]
hash/default/str_64     time:   [13.765 ns 13.807 ns 13.859 ns]
                        thrpt:  [4.3008 GiB/s 4.3168 GiB/s 4.3300 GiB/s]
hash/default/str_100    time:   [18.527 ns 18.543 ns 18.562 ns]
                        thrpt:  [5.0175 GiB/s 5.0225 GiB/s 5.0268 GiB/s]
hash/default/str_177    time:   [31.003 ns 31.041 ns 31.082 ns]
                        thrpt:  [5.3035 GiB/s 5.3106 GiB/s 5.3170 GiB/s]
hash/default/str_256    time:   [39.984 ns 40.101 ns 40.245 ns]
                        thrpt:  [5.9242 GiB/s 5.9454 GiB/s 5.9628 GiB/s]
hash/default/str_1024   time:   [149.87 ns 150.11 ns 150.37 ns]
                        thrpt:  [6.3422 GiB/s 6.3530 GiB/s 6.3632 GiB/s]
hash/default/str_4096   time:   [576.78 ns 577.65 ns 578.61 ns]
                        thrpt:  [6.5928 GiB/s 6.6039 GiB/s 6.6137 GiB/s]
hash/default/u64        time:   [8.6986 ns 8.7084 ns 8.7201 ns]
                        thrpt:  [114.68 Melem/s 114.83 Melem/s 114.96 Melem/s]
hash/default/object     time:   [35.229 ns 35.279 ns 35.340 ns]
                        thrpt:  [28.297 Melem/s 28.346 Melem/s 28.385 Melem/s]

hash/fxhash/str_2       time:   [1.5646 ns 1.5680 ns 1.5719 ns]
                        thrpt:  [1.1849 GiB/s 1.1879 GiB/s 1.1905 GiB/s]
hash/fxhash/str_8       time:   [1.3512 ns 1.3550 ns 1.3597 ns]
                        thrpt:  [5.4796 GiB/s 5.4987 GiB/s 5.5142 GiB/s]
hash/fxhash/str_16      time:   [1.7534 ns 1.7662 ns 1.7782 ns]
                        thrpt:  [8.3799 GiB/s 8.4370 GiB/s 8.4987 GiB/s]
hash/fxhash/str_64      time:   [3.8118 ns 3.8253 ns 3.8425 ns]
                        thrpt:  [15.512 GiB/s 15.582 GiB/s 15.637 GiB/s]
hash/fxhash/str_100     time:   [6.7739 ns 6.8098 ns 6.8589 ns]
                        thrpt:  [13.578 GiB/s 13.676 GiB/s 13.749 GiB/s]
hash/fxhash/str_177     time:   [14.594 ns 14.617 ns 14.644 ns]
                        thrpt:  [11.257 GiB/s 11.278 GiB/s 11.296 GiB/s]
hash/fxhash/str_256     time:   [20.954 ns 21.004 ns 21.050 ns]
                        thrpt:  [11.326 GiB/s 11.351 GiB/s 11.378 GiB/s]
hash/fxhash/str_1024    time:   [123.11 ns 123.32 ns 123.58 ns]
                        thrpt:  [7.7173 GiB/s 7.7335 GiB/s 7.7464 GiB/s]
hash/fxhash/str_4096    time:   [549.18 ns 549.44 ns 549.73 ns]
                        thrpt:  [6.9392 GiB/s 6.9429 GiB/s 6.9462 GiB/s]
hash/fxhash/u64         time:   [270.19 ps 280.82 ps 290.90 ps]
                        thrpt:  [3.4376 Gelem/s 3.5610 Gelem/s 3.7011 Gelem/s]
hash/fxhash/object      time:   [7.4055 ns 7.4255 ns 7.4520 ns]
                        thrpt:  [134.19 Melem/s 134.67 Melem/s 135.04 Melem/s]

hash/gxhash/str_2       time:   [2.0821 ns 2.0916 ns 2.1004 ns]
                        thrpt:  [908.07 MiB/s 911.92 MiB/s 916.06 MiB/s]
hash/gxhash/str_8       time:   [2.0697 ns 2.0756 ns 2.0814 ns]
                        thrpt:  [3.5797 GiB/s 3.5895 GiB/s 3.5999 GiB/s]
hash/gxhash/str_16      time:   [2.0709 ns 2.0781 ns 2.0853 ns]
                        thrpt:  [7.1457 GiB/s 7.1705 GiB/s 7.1954 GiB/s]
hash/gxhash/str_64      time:   [2.5302 ns 2.5363 ns 2.5433 ns]
                        thrpt:  [23.436 GiB/s 23.500 GiB/s 23.557 GiB/s]
hash/gxhash/str_100     time:   [4.3518 ns 4.3638 ns 4.3777 ns]
                        thrpt:  [21.274 GiB/s 21.342 GiB/s 21.401 GiB/s]
hash/gxhash/str_177     time:   [5.1608 ns 5.3059 ns 5.4850 ns]
                        thrpt:  [30.054 GiB/s 31.068 GiB/s 31.942 GiB/s]
hash/gxhash/str_256     time:   [5.8449 ns 5.9005 ns 5.9631 ns]
                        thrpt:  [39.982 GiB/s 40.406 GiB/s 40.791 GiB/s]
hash/gxhash/str_1024    time:   [16.506 ns 16.708 ns 16.947 ns]
                        thrpt:  [56.275 GiB/s 57.079 GiB/s 57.776 GiB/s]
hash/gxhash/str_4096    time:   [53.849 ns 54.332 ns 54.944 ns]
                        thrpt:  [69.429 GiB/s 70.210 GiB/s 70.840 GiB/s]
hash/gxhash/u64         time:   [537.11 ps 539.49 ps 541.98 ps]
                        thrpt:  [1.8451 Gelem/s 1.8536 Gelem/s 1.8618 Gelem/s]
hash/gxhash/object      time:   [6.2974 ns 6.3124 ns 6.3290 ns]
                        thrpt:  [158.00 Melem/s 158.42 Melem/s 158.80 Melem/s]

hash/ahash/str_2        time:   [3.5844 ns 3.5928 ns 3.6034 ns]
                        thrpt:  [529.32 MiB/s 530.88 MiB/s 532.12 MiB/s]
hash/ahash/str_8        time:   [3.5813 ns 3.5861 ns 3.5914 ns]
                        thrpt:  [2.0745 GiB/s 2.0776 GiB/s 2.0804 GiB/s]
hash/ahash/str_16       time:   [3.8121 ns 3.8230 ns 3.8357 ns]
                        thrpt:  [3.8849 GiB/s 3.8977 GiB/s 3.9089 GiB/s]
hash/ahash/str_64       time:   [5.5384 ns 5.5478 ns 5.5582 ns]
                        thrpt:  [10.724 GiB/s 10.744 GiB/s 10.762 GiB/s]
hash/ahash/str_100      time:   [12.679 ns 12.705 ns 12.736 ns]
                        thrpt:  [7.3128 GiB/s 7.3302 GiB/s 7.3454 GiB/s]
hash/ahash/str_177      time:   [16.211 ns 16.250 ns 16.305 ns]
                        thrpt:  [10.110 GiB/s 10.144 GiB/s 10.168 GiB/s]
hash/ahash/str_256      time:   [20.342 ns 20.437 ns 20.542 ns]
                        thrpt:  [11.607 GiB/s 11.666 GiB/s 11.720 GiB/s]
hash/ahash/str_1024     time:   [87.360 ns 87.595 ns 87.873 ns]
                        thrpt:  [10.853 GiB/s 10.887 GiB/s 10.917 GiB/s]
hash/ahash/str_4096     time:   [357.18 ns 357.70 ns 358.50 ns]
                        thrpt:  [10.641 GiB/s 10.664 GiB/s 10.680 GiB/s]
hash/ahash/u64          time:   [2.4700 ns 2.4708 ns 2.4718 ns]
                        thrpt:  [404.56 Melem/s 404.72 Melem/s 404.85 Melem/s]
hash/ahash/object       time:   [10.053 ns 10.096 ns 10.153 ns]
                        thrpt:  [98.491 Melem/s 99.049 Melem/s 99.471 Melem/s]

hash/t1ha/str_2         time:   [4.6847 ns 4.7004 ns 4.7173 ns]
                        thrpt:  [404.33 MiB/s 405.79 MiB/s 407.15 MiB/s]
hash/t1ha/str_8         time:   [4.6528 ns 4.6695 ns 4.6867 ns]
                        thrpt:  [1.5897 GiB/s 1.5956 GiB/s 1.6013 GiB/s]
hash/t1ha/str_16        time:   [4.4657 ns 4.4691 ns 4.4725 ns]
                        thrpt:  [3.3317 GiB/s 3.3343 GiB/s 3.3368 GiB/s]
hash/t1ha/str_64        time:   [7.8642 ns 7.8741 ns 7.8848 ns]
                        thrpt:  [7.5595 GiB/s 7.5697 GiB/s 7.5792 GiB/s]
hash/t1ha/str_100       time:   [9.4759 ns 9.5030 ns 9.5326 ns]
                        thrpt:  [9.7699 GiB/s 9.8003 GiB/s 9.8283 GiB/s]
hash/t1ha/str_177       time:   [12.178 ns 12.202 ns 12.231 ns]
                        thrpt:  [13.477 GiB/s 13.510 GiB/s 13.537 GiB/s]
hash/t1ha/str_256       time:   [15.901 ns 15.918 ns 15.937 ns]
                        thrpt:  [14.960 GiB/s 14.978 GiB/s 14.994 GiB/s]
hash/t1ha/str_1024      time:   [51.577 ns 51.757 ns 51.978 ns]
                        thrpt:  [18.348 GiB/s 18.426 GiB/s 18.490 GiB/s]
hash/t1ha/str_4096      time:   [204.82 ns 206.03 ns 208.39 ns]
                        thrpt:  [18.306 GiB/s 18.515 GiB/s 18.625 GiB/s]
hash/t1ha/u64           time:   [4.8599 ns 4.8859 ns 4.9126 ns]
                        thrpt:  [203.56 Melem/s 204.67 Melem/s 205.77 Melem/s]
hash/t1ha/object        time:   [29.165 ns 29.239 ns 29.342 ns]
                        thrpt:  [34.081 Melem/s 34.201 Melem/s 34.287 Melem/s]

hash/wyhash/str_2       time:   [3.5889 ns 3.5998 ns 3.6134 ns]
                        thrpt:  [527.86 MiB/s 529.85 MiB/s 531.45 MiB/s]
hash/wyhash/str_8       time:   [3.5692 ns 3.5795 ns 3.5931 ns]
                        thrpt:  [2.0736 GiB/s 2.0815 GiB/s 2.0874 GiB/s]
hash/wyhash/str_16      time:   [3.7987 ns 3.8129 ns 3.8290 ns]
                        thrpt:  [3.8917 GiB/s 3.9081 GiB/s 3.9227 GiB/s]
hash/wyhash/str_64      time:   [4.7435 ns 4.7759 ns 4.8133 ns]
                        thrpt:  [12.383 GiB/s 12.480 GiB/s 12.566 GiB/s]
hash/wyhash/str_100     time:   [7.3236 ns 7.3413 ns 7.3625 ns]
                        thrpt:  [12.650 GiB/s 12.686 GiB/s 12.717 GiB/s]
hash/wyhash/str_177     time:   [11.441 ns 11.465 ns 11.489 ns]
                        thrpt:  [14.348 GiB/s 14.379 GiB/s 14.409 GiB/s]
hash/wyhash/str_256     time:   [13.878 ns 13.900 ns 13.926 ns]
                        thrpt:  [17.120 GiB/s 17.153 GiB/s 17.180 GiB/s]
hash/wyhash/str_1024    time:   [52.311 ns 52.579 ns 52.819 ns]
                        thrpt:  [18.055 GiB/s 18.138 GiB/s 18.231 GiB/s]
hash/wyhash/str_4096    time:   [204.49 ns 204.68 ns 204.87 ns]
                        thrpt:  [18.620 GiB/s 18.637 GiB/s 18.655 GiB/s]
hash/wyhash/u64         time:   [816.77 ps 819.46 ps 822.38 ps]
                        thrpt:  [1.2160 Gelem/s 1.2203 Gelem/s 1.2243 Gelem/s]
hash/wyhash/object      time:   [14.395 ns 14.445 ns 14.495 ns]
                        thrpt:  [68.990 Melem/s 69.228 Melem/s 69.469 Melem/s]

hash/wyhash_raw/str_2   time:   [3.1420 ns 3.1507 ns 3.1607 ns]
                        thrpt:  [603.46 MiB/s 605.37 MiB/s 607.06 MiB/s]
hash/wyhash_raw/str_8   time:   [3.1600 ns 3.1937 ns 3.2367 ns]
                        thrpt:  [2.3019 GiB/s 2.3329 GiB/s 2.3578 GiB/s]
hash/wyhash_raw/str_16  time:   [3.2794 ns 3.2875 ns 3.2966 ns]
                        thrpt:  [4.5202 GiB/s 4.5327 GiB/s 4.5439 GiB/s]
hash/wyhash_raw/str_64  time:   [4.0505 ns 4.0621 ns 4.0759 ns]
                        thrpt:  [14.624 GiB/s 14.673 GiB/s 14.715 GiB/s]
hash/wyhash_raw/str_100 time:   [6.9784 ns 6.9947 ns 7.0119 ns]
                        thrpt:  [13.282 GiB/s 13.315 GiB/s 13.346 GiB/s]
hash/wyhash_raw/str_177 time:   [10.150 ns 10.187 ns 10.222 ns]
                        thrpt:  [16.127 GiB/s 16.182 GiB/s 16.240 GiB/s]
hash/wyhash_raw/str_256 time:   [12.861 ns 12.956 ns 13.113 ns]
                        thrpt:  [18.182 GiB/s 18.402 GiB/s 18.538 GiB/s]
hash/wyhash_raw/str_1024
                        time:   [47.880 ns 47.936 ns 47.994 ns]
                        thrpt:  [19.871 GiB/s 19.895 GiB/s 19.918 GiB/s]
hash/wyhash_raw/str_4096
                        time:   [193.48 ns 193.76 ns 194.04 ns]
                        thrpt:  [19.659 GiB/s 19.688 GiB/s 19.716 GiB/s]
hash/wyhash_raw/u64     time:   [3.3909 ns 3.4046 ns 3.4234 ns]
                        thrpt:  [292.11 Melem/s 293.72 Melem/s 294.91 Melem/s]

hash/xxhash/str_2       time:   [8.6982 ns 8.7088 ns 8.7228 ns]
                        thrpt:  [218.66 MiB/s 219.01 MiB/s 219.28 MiB/s]
hash/xxhash/str_8       time:   [8.0113 ns 8.0176 ns 8.0249 ns]
                        thrpt:  [950.72 MiB/s 951.58 MiB/s 952.33 MiB/s]
hash/xxhash/str_16      time:   [8.6008 ns 8.6261 ns 8.6535 ns]
                        thrpt:  [1.7220 GiB/s 1.7274 GiB/s 1.7325 GiB/s]
hash/xxhash/str_64      time:   [12.532 ns 12.547 ns 12.565 ns]
                        thrpt:  [4.7438 GiB/s 4.7504 GiB/s 4.7560 GiB/s]
hash/xxhash/str_100     time:   [14.241 ns 14.306 ns 14.378 ns]
                        thrpt:  [6.4775 GiB/s 6.5100 GiB/s 6.5397 GiB/s]
hash/xxhash/str_177     time:   [19.877 ns 19.910 ns 19.952 ns]
                        thrpt:  [8.2620 GiB/s 8.2796 GiB/s 8.2933 GiB/s]
hash/xxhash/str_256     time:   [22.254 ns 22.276 ns 22.297 ns]
                        thrpt:  [10.693 GiB/s 10.703 GiB/s 10.714 GiB/s]
hash/xxhash/str_1024    time:   [66.297 ns 66.605 ns 66.920 ns]
                        thrpt:  [14.251 GiB/s 14.318 GiB/s 14.385 GiB/s]
hash/xxhash/str_4096    time:   [234.93 ns 235.66 ns 236.42 ns]
                        thrpt:  [16.136 GiB/s 16.187 GiB/s 16.238 GiB/s]
hash/xxhash/u64         time:   [8.5823 ns 8.5840 ns 8.5860 ns]
                        thrpt:  [116.47 Melem/s 116.50 Melem/s 116.52 Melem/s]
hash/xxhash/object      time:   [40.945 ns 40.991 ns 41.048 ns]
                        thrpt:  [24.362 Melem/s 24.396 Melem/s 24.423 Melem/s]

hash/metrohash/str_2    time:   [3.9937 ns 4.0221 ns 4.0559 ns]
                        thrpt:  [470.26 MiB/s 474.22 MiB/s 477.59 MiB/s]
hash/metrohash/str_8    time:   [3.5995 ns 3.6081 ns 3.6180 ns]
                        thrpt:  [2.0593 GiB/s 2.0649 GiB/s 2.0699 GiB/s]
hash/metrohash/str_16   time:   [4.4729 ns 4.4857 ns 4.5002 ns]
                        thrpt:  [3.3112 GiB/s 3.3219 GiB/s 3.3314 GiB/s]
hash/metrohash/str_64   time:   [7.9725 ns 8.0097 ns 8.0525 ns]
                        thrpt:  [7.4020 GiB/s 7.4415 GiB/s 7.4763 GiB/s]
hash/metrohash/str_100  time:   [10.846 ns 10.868 ns 10.893 ns]
                        thrpt:  [8.5496 GiB/s 8.5695 GiB/s 8.5866 GiB/s]
hash/metrohash/str_177  time:   [16.730 ns 16.747 ns 16.768 ns]
                        thrpt:  [9.8310 GiB/s 9.8431 GiB/s 9.8535 GiB/s]
hash/metrohash/str_256  time:   [18.075 ns 18.101 ns 18.141 ns]
                        thrpt:  [13.143 GiB/s 13.171 GiB/s 13.191 GiB/s]
hash/metrohash/str_1024 time:   [54.896 ns 54.972 ns 55.055 ns]
                        thrpt:  [17.322 GiB/s 17.348 GiB/s 17.372 GiB/s]
hash/metrohash/str_4096 time:   [201.59 ns 202.29 ns 203.31 ns]
                        thrpt:  [18.763 GiB/s 18.857 GiB/s 18.923 GiB/s]
hash/metrohash/u64      time:   [1.0611 ns 1.0643 ns 1.0696 ns]
                        thrpt:  [934.95 Melem/s 939.55 Melem/s 942.45 Melem/s]
hash/metrohash/object   time:   [37.461 ns 37.651 ns 37.890 ns]
                        thrpt:  [26.392 Melem/s 26.559 Melem/s 26.694 Melem/s]

hash/seahash/str_2      time:   [13.204 ns 13.226 ns 13.248 ns]
                        thrpt:  [143.97 MiB/s 144.21 MiB/s 144.45 MiB/s]
hash/seahash/str_8      time:   [8.8871 ns 8.9111 ns 8.9464 ns]
                        thrpt:  [852.79 MiB/s 856.17 MiB/s 858.48 MiB/s]
hash/seahash/str_16     time:   [9.6197 ns 9.6540 ns 9.6978 ns]
                        thrpt:  [1.5365 GiB/s 1.5435 GiB/s 1.5490 GiB/s]
hash/seahash/str_64     time:   [13.672 ns 13.688 ns 13.708 ns]
                        thrpt:  [4.3481 GiB/s 4.3545 GiB/s 4.3595 GiB/s]
hash/seahash/str_100    time:   [16.890 ns 16.915 ns 16.946 ns]
                        thrpt:  [5.4959 GiB/s 5.5060 GiB/s 5.5142 GiB/s]
hash/seahash/str_177    time:   [25.372 ns 25.388 ns 25.406 ns]
                        thrpt:  [6.4884 GiB/s 6.4929 GiB/s 6.4972 GiB/s]
hash/seahash/str_256    time:   [31.874 ns 31.932 ns 31.996 ns]
                        thrpt:  [7.4515 GiB/s 7.4665 GiB/s 7.4799 GiB/s]
hash/seahash/str_1024   time:   [114.42 ns 114.53 ns 114.63 ns]
                        thrpt:  [8.3194 GiB/s 8.3271 GiB/s 8.3346 GiB/s]
hash/seahash/str_4096   time:   [426.70 ns 427.51 ns 428.56 ns]
                        thrpt:  [8.9011 GiB/s 8.9230 GiB/s 8.9401 GiB/s]
hash/seahash/u64        time:   [8.4910 ns 8.5340 ns 8.5873 ns]
                        thrpt:  [116.45 Melem/s 117.18 Melem/s 117.77 Melem/s]
hash/seahash/object     time:   [68.622 ns 68.823 ns 69.016 ns]
                        thrpt:  [14.489 Melem/s 14.530 Melem/s 14.573 Melem/s]

hash/farmhash/str_2     time:   [12.217 ns 12.240 ns 12.265 ns]
                        thrpt:  [155.51 MiB/s 155.83 MiB/s 156.12 MiB/s]
hash/farmhash/str_8     time:   [15.001 ns 15.096 ns 15.202 ns]
                        thrpt:  [501.85 MiB/s 505.40 MiB/s 508.60 MiB/s]
hash/farmhash/str_16    time:   [15.365 ns 15.401 ns 15.438 ns]
                        thrpt:  [988.42 MiB/s 990.79 MiB/s 993.10 MiB/s]
hash/farmhash/str_64    time:   [34.148 ns 34.210 ns 34.314 ns]
                        thrpt:  [1.7370 GiB/s 1.7423 GiB/s 1.7455 GiB/s]
hash/farmhash/str_100   time:   [45.195 ns 45.334 ns 45.520 ns]
                        thrpt:  [2.0460 GiB/s 2.0544 GiB/s 2.0607 GiB/s]
hash/farmhash/str_177   time:   [58.431 ns 58.656 ns 58.933 ns]
                        thrpt:  [2.7971 GiB/s 2.8103 GiB/s 2.8212 GiB/s]
hash/farmhash/str_256   time:   [61.664 ns 61.791 ns 61.966 ns]
                        thrpt:  [3.8476 GiB/s 3.8585 GiB/s 3.8664 GiB/s]
hash/farmhash/str_1024  time:   [102.98 ns 103.12 ns 103.29 ns]
                        thrpt:  [9.2326 GiB/s 9.2479 GiB/s 9.2609 GiB/s]
hash/farmhash/str_4096  time:   [274.91 ns 275.50 ns 276.18 ns]
                        thrpt:  [13.812 GiB/s 13.846 GiB/s 13.876 GiB/s]
hash/farmhash/u64       time:   [10.040 ns 10.086 ns 10.136 ns]
                        thrpt:  [98.662 Melem/s 99.146 Melem/s 99.600 Melem/s]
hash/farmhash/object    time:   [100.89 ns 101.06 ns 101.25 ns]
                        thrpt:  [9.8767 Melem/s 9.8955 Melem/s 9.9119 Melem/s]

hash/highwayhash/str_2  time:   [33.968 ns 33.979 ns 33.990 ns]
                        thrpt:  [56.115 MiB/s 56.134 MiB/s 56.151 MiB/s]
hash/highwayhash/str_8  time:   [32.579 ns 32.629 ns 32.691 ns]
                        thrpt:  [233.38 MiB/s 233.82 MiB/s 234.18 MiB/s]
hash/highwayhash/str_16 time:   [29.252 ns 29.312 ns 29.367 ns]
                        thrpt:  [519.59 MiB/s 520.56 MiB/s 521.63 MiB/s]
hash/highwayhash/str_64 time:   [31.436 ns 31.601 ns 31.749 ns]
                        thrpt:  [1.8774 GiB/s 1.8862 GiB/s 1.8961 GiB/s]
hash/highwayhash/str_100
                        time:   [39.807 ns 39.859 ns 39.922 ns]
                        thrpt:  [2.3328 GiB/s 2.3366 GiB/s 2.3396 GiB/s]
hash/highwayhash/str_177
                        time:   [43.941 ns 44.027 ns 44.130 ns]
                        thrpt:  [3.7355 GiB/s 3.7441 GiB/s 3.7515 GiB/s]
hash/highwayhash/str_256
                        time:   [41.693 ns 41.783 ns 41.871 ns]
                        thrpt:  [5.6941 GiB/s 5.7061 GiB/s 5.7185 GiB/s]
hash/highwayhash/str_1024
                        time:   [84.306 ns 84.367 ns 84.433 ns]
                        thrpt:  [11.295 GiB/s 11.304 GiB/s 11.312 GiB/s]
hash/highwayhash/str_4096
                        time:   [250.64 ns 251.55 ns 252.89 ns]
                        thrpt:  [15.085 GiB/s 15.164 GiB/s 15.220 GiB/s]
hash/highwayhash/u64    time:   [34.785 ns 34.846 ns 34.917 ns]
                        thrpt:  [28.639 Melem/s 28.698 Melem/s 28.748 Melem/s]
hash/highwayhash/object time:   [52.139 ns 52.216 ns 52.296 ns]
                        thrpt:  [19.122 Melem/s 19.151 Melem/s 19.179 Melem/s]

hash/rustc-hash/str_2   time:   [2.2361 ns 2.2436 ns 2.2528 ns]
                        thrpt:  [846.64 MiB/s 850.13 MiB/s 852.99 MiB/s]
hash/rustc-hash/str_8   time:   [2.0174 ns 2.0266 ns 2.0387 ns]
                        thrpt:  [3.6546 GiB/s 3.6764 GiB/s 3.6931 GiB/s]
hash/rustc-hash/str_16  time:   [1.9798 ns 1.9919 ns 2.0092 ns]
                        thrpt:  [7.4165 GiB/s 7.4808 GiB/s 7.5267 GiB/s]
hash/rustc-hash/str_64  time:   [3.4341 ns 3.4579 ns 3.4845 ns]
                        thrpt:  [17.106 GiB/s 17.237 GiB/s 17.357 GiB/s]
hash/rustc-hash/str_100 time:   [4.6333 ns 4.6650 ns 4.7019 ns]
                        thrpt:  [19.807 GiB/s 19.964 GiB/s 20.101 GiB/s]
hash/rustc-hash/str_177 time:   [7.3716 ns 7.4194 ns 7.4696 ns]
                        thrpt:  [22.069 GiB/s 22.218 GiB/s 22.362 GiB/s]
hash/rustc-hash/str_256 time:   [9.4740 ns 9.6159 ns 9.8823 ns]
                        thrpt:  [24.126 GiB/s 24.794 GiB/s 25.166 GiB/s]
hash/rustc-hash/str_1024
                        time:   [39.309 ns 39.640 ns 40.197 ns]
                        thrpt:  [23.725 GiB/s 24.059 GiB/s 24.261 GiB/s]
hash/rustc-hash/str_4096
                        time:   [174.53 ns 175.03 ns 175.46 ns]
                        thrpt:  [21.741 GiB/s 21.795 GiB/s 21.857 GiB/s]
hash/rustc-hash/u64     time:   [389.99 ps 403.97 ps 425.58 ps]
                        thrpt:  [2.3497 Gelem/s 2.4755 Gelem/s 2.5642 Gelem/s]
hash/rustc-hash/object  time:   [4.3501 ns 4.3568 ns 4.3652 ns]
                        thrpt:  [229.09 Melem/s 229.52 Melem/s 229.88 Melem/s]

map/rapidhash/1000_small
                        time:   [41.712 µs 41.778 µs 41.860 µs]
                        thrpt:  [23.889 Melem/s 23.936 Melem/s 23.974 Melem/s]
map/rapidhash/10000_emails
                        time:   [466.07 µs 466.93 µs 467.90 µs]
                        thrpt:  [21.372 Melem/s 21.416 Melem/s 21.456 Melem/s]
map/rapidhash/450000_words
                        time:   [63.365 ms 63.982 ms 64.619 ms]
                        thrpt:  [6.9639 Melem/s 7.0332 Melem/s 7.1018 Melem/s]
map/rapidhash/100000_u64
                        time:   [1.5572 ms 1.5599 ms 1.5633 ms]
                        thrpt:  [63.969 Melem/s 64.105 Melem/s 64.218 Melem/s]
map/rapidhash/10000_struct
                        time:   [1.7165 ms 1.7333 ms 1.7545 ms]
                        thrpt:  [5.6997 Melem/s 5.7694 Melem/s 5.8259 Melem/s]

map/rapidhash_inline/1000_small
                        time:   [45.285 µs 45.610 µs 45.979 µs]
                        thrpt:  [21.749 Melem/s 21.925 Melem/s 22.083 Melem/s]
map/rapidhash_inline/10000_emails
                        time:   [475.34 µs 476.56 µs 477.89 µs]
                        thrpt:  [20.925 Melem/s 20.984 Melem/s 21.038 Melem/s]
map/rapidhash_inline/450000_words
                        time:   [66.242 ms 67.037 ms 67.883 ms]
                        thrpt:  [6.6291 Melem/s 6.7127 Melem/s 6.7932 Melem/s]
map/rapidhash_inline/100000_u64
                        time:   [1.5311 ms 1.5337 ms 1.5368 ms]
                        thrpt:  [65.071 Melem/s 65.201 Melem/s 65.313 Melem/s]
map/rapidhash_inline/10000_struct
                        time:   [1.7286 ms 1.7573 ms 1.7909 ms]
                        thrpt:  [5.5837 Melem/s 5.6906 Melem/s 5.7849 Melem/s]

map/default/1000_small  time:   [54.549 µs 54.687 µs 54.867 µs]
                        thrpt:  [18.226 Melem/s 18.286 Melem/s 18.332 Melem/s]
map/default/10000_emails
                        time:   [777.28 µs 778.47 µs 779.71 µs]
                        thrpt:  [12.825 Melem/s 12.846 Melem/s 12.865 Melem/s]
map/default/450000_words
                        time:   [89.540 ms 90.926 ms 92.450 ms]
                        thrpt:  [4.8675 Melem/s 4.9491 Melem/s 5.0257 Melem/s]
map/default/100000_u64  time:   [2.9957 ms 3.0022 ms 3.0128 ms]
                        thrpt:  [33.192 Melem/s 33.309 Melem/s 33.382 Melem/s]
map/default/10000_struct
                        time:   [3.7879 ms 3.8101 ms 3.8361 ms]
                        thrpt:  [2.6068 Melem/s 2.6246 Melem/s 2.6400 Melem/s]

map/fxhash/1000_small   time:   [32.432 µs 32.525 µs 32.636 µs]
                        thrpt:  [30.641 Melem/s 30.745 Melem/s 30.834 Melem/s]
map/fxhash/10000_emails time:   [507.89 µs 510.30 µs 512.70 µs]
                        thrpt:  [19.505 Melem/s 19.596 Melem/s 19.689 Melem/s]
map/fxhash/450000_words time:   [62.584 ms 63.327 ms 64.109 ms]
                        thrpt:  [7.0193 Melem/s 7.1060 Melem/s 7.1904 Melem/s]
map/fxhash/100000_u64   time:   [1.4470 ms 1.4481 ms 1.4495 ms]
                        thrpt:  [68.988 Melem/s 69.055 Melem/s 69.110 Melem/s]
map/fxhash/10000_struct time:   [2.4016 ms 2.4135 ms 2.4297 ms]
                        thrpt:  [4.1157 Melem/s 4.1434 Melem/s 4.1639 Melem/s]

map/gxhash/1000_small   time:   [48.947 µs 48.964 µs 48.984 µs]
                        thrpt:  [20.415 Melem/s 20.423 Melem/s 20.430 Melem/s]
map/gxhash/10000_emails time:   [551.52 µs 552.04 µs 552.82 µs]
                        thrpt:  [18.089 Melem/s 18.115 Melem/s 18.132 Melem/s]
map/gxhash/450000_words time:   [66.830 ms 67.681 ms 68.638 ms]
                        thrpt:  [6.5562 Melem/s 6.6489 Melem/s 6.7335 Melem/s]
map/gxhash/100000_u64   time:   [1.8809 ms 1.8838 ms 1.8871 ms]
                        thrpt:  [52.991 Melem/s 53.085 Melem/s 53.165 Melem/s]
map/gxhash/10000_struct time:   [1.6558 ms 1.6735 ms 1.6963 ms]
                        thrpt:  [5.8951 Melem/s 5.9755 Melem/s 6.0394 Melem/s]

map/wyhash/1000_small   time:   [40.754 µs 40.774 µs 40.795 µs]
                        thrpt:  [24.513 Melem/s 24.525 Melem/s 24.537 Melem/s]
map/wyhash/10000_emails time:   [593.33 µs 594.52 µs 595.50 µs]
                        thrpt:  [16.793 Melem/s 16.820 Melem/s 16.854 Melem/s]
map/wyhash/450000_words time:   [71.123 ms 71.943 ms 72.820 ms]
                        thrpt:  [6.1796 Melem/s 6.2550 Melem/s 6.3271 Melem/s]
map/wyhash/100000_u64   time:   [1.5850 ms 1.5913 ms 1.5984 ms]
                        thrpt:  [62.563 Melem/s 62.841 Melem/s 63.093 Melem/s]
map/wyhash/10000_struct time:   [2.0702 ms 2.0890 ms 2.1127 ms]
                        thrpt:  [4.7332 Melem/s 4.7869 Melem/s 4.8305 Melem/s]

rng/rapidhash/1         time:   [1.5777 ns 1.5802 ns 1.5827 ns]
                        thrpt:  [631.82 Melem/s 632.83 Melem/s 633.81 Melem/s]
rng/rapidhash/10000     time:   [6.0007 µs 6.0124 µs 6.0306 µs]
                        thrpt:  [1.6582 Gelem/s 1.6632 Gelem/s 1.6665 Gelem/s]

rng/rapidhash_fast/1    time:   [1.0501 ns 1.0520 ns 1.0545 ns]
                        thrpt:  [948.32 Melem/s 950.60 Melem/s 952.27 Melem/s]
rng/rapidhash_fast/10000
                        time:   [5.7921 µs 5.8047 µs 5.8203 µs]
                        thrpt:  [1.7181 Gelem/s 1.7227 Gelem/s 1.7265 Gelem/s]

rng/rapidhash_time/1    time:   [66.295 ns 66.342 ns 66.403 ns]
                        thrpt:  [15.060 Melem/s 15.073 Melem/s 15.084 Melem/s]
rng/rapidhash_time/10000
                        time:   [313.97 µs 314.67 µs 315.55 µs]
                        thrpt:  [31.691 Melem/s 31.779 Melem/s 31.850 Melem/s]

rng/wyhash/1            time:   [1.5492 ns 1.5544 ns 1.5601 ns]
                        thrpt:  [640.98 Melem/s 643.35 Melem/s 645.48 Melem/s]
rng/wyhash/10000        time:   [6.3016 µs 6.3220 µs 6.3447 µs]
                        thrpt:  [1.5761 Gelem/s 1.5818 Gelem/s 1.5869 Gelem/s]

compiled/match_hash     time:   [18.631 ns 18.638 ns 18.645 ns]
compiled/match_slice    time:   [18.720 ns 18.758 ns 18.804 ns]
compiled/hashmap_get    time:   [12.806 ns 12.816 ns 12.830 ns]
```

Afterward, I PR'd an extra test:

```
hash/xxhash3_64/str_2   time:   [21.819 ns 21.879 ns 21.958 ns]
                        thrpt:  [86.864 MiB/s 87.179 MiB/s 87.416 MiB/s]
hash/xxhash3_64/str_8   time:   [21.744 ns 21.775 ns 21.810 ns]
                        thrpt:  [349.80 MiB/s 350.37 MiB/s 350.88 MiB/s]
hash/xxhash3_64/str_16  time:   [21.507 ns 21.534 ns 21.573 ns]
                        thrpt:  [707.32 MiB/s 708.58 MiB/s 709.47 MiB/s]
hash/xxhash3_64/str_64  time:   [23.251 ns 23.275 ns 23.306 ns]
                        thrpt:  [2.5575 GiB/s 2.5609 GiB/s 2.5635 GiB/s]
hash/xxhash3_64/str_100 time:   [27.651 ns 27.721 ns 27.801 ns]
                        thrpt:  [3.3500 GiB/s 3.3596 GiB/s 3.3682 GiB/s]
hash/xxhash3_64/str_177 time:   [33.251 ns 33.273 ns 33.299 ns]
                        thrpt:  [4.9504 GiB/s 4.9542 GiB/s 4.9576 GiB/s]
hash/xxhash3_64/str_256 time:   [28.331 ns 28.407 ns 28.494 ns]
                        thrpt:  [8.3673 GiB/s 8.3931 GiB/s 8.4155 GiB/s]
hash/xxhash3_64/str_1024
                        time:   [48.690 ns 48.786 ns 48.889 ns]
                        thrpt:  [19.507 GiB/s 19.548 GiB/s 19.587 GiB/s]
hash/xxhash3_64/str_4096
                        time:   [95.395 ns 96.063 ns 96.790 ns]
                        thrpt:  [39.412 GiB/s 39.710 GiB/s 39.988 GiB/s]
hash/xxhash3_64/u64     time:   [21.215 ns 21.242 ns 21.269 ns]
                        thrpt:  [47.016 Melem/s 47.077 Melem/s 47.137 Melem/s]
hash/xxhash3_64/object  time:   [46.166 ns 46.267 ns 46.417 ns]
                        thrpt:  [21.544 Melem/s 21.614 Melem/s 21.661 Melem/s]
```

[rapidhash-testsuite]: https://github.com/hoxxep/rapidhash/tree/master/benches