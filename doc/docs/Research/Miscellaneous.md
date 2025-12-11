# Chunk Size

For this I tested with `-Skyrim 202X 10.0.1 - Architecture PART 1-2347-10-0-1710488193`.
I removed all non regular textures, i.e. those ending with `_n.dds` and `_p.dds` as most game mods
don't ship normals etc.

Results:

| Block Size   | File Size (MiB) |
| ------------ | --------------- |
| Uncompressed | 9695.84         |
| 128K         | 7739.02         |
| 1M           | 7373.02         |
| 4M           | 7097.22         |
| 8M           | 7007.22         |
| 16M          | 6965.84         |
| 32M          | 6946.49         |
| 64M          | 6938.88         |
| 256M         | 6934.86         |

File Size Distribution:

| Row | Start    | End      | Files |
| --- | -------- | -------- | ----- |
| 1   | 21.5 kB  | 14.2 MB  | 101   |
| 2   | 14.2 MB  | 28.5 MB  | 40    |
| 3   | 28.5 MB  | 42.7 MB  | 52    |
| 4   | 42.7 MB  | 56.9 MB  | 0     |
| 5   | 56.9 MB  | 71.1 MB  | 0     |
| 6   | 71.1 MB  | 85.3 MB  | 68    |
| 7   | 85.3 MB  | 99.6 MB  | 0     |
| 8   | 99.6 MB  | 113.8 MB | 0     |
| 9   | 113.8 MB | 128.0 MB | 0     |
| 10  | 128.0 MB | 142.2 MB | 0     |
| 11  | 142.2 MB | 156.4 MB | 0     |
| 12  | 156.4 MB | 170.7 MB | 1     |