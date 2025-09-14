# TinyMalloc Segment Space Utilization Analysis

## Test Configuration
- **Segment size**: 131,072 bytes (128 KB)
- **Total size classes**: 32
- **Word size**: 8 bytes (64-bit architecture)

## Individual Class Metrics

| Class | Object Size | Max Objects | Wasted Bytes | Utilization | Bitmap Words |
|-------|-------------|-------------|--------------|-------------|--------------|
| 0     | 8           | 16,119      | 0            | 100.0%      | 256          |
| 1     | 16          | 8,123       | 8            | 100.0%      | 128          |
| 2     | 24          | 5,429       | 16           | 100.0%      | 86           |
| 3     | 32          | 4,077       | 24           | 100.0%      | 64           |
| 4     | 40          | 3,264       | 24           | 100.0%      | 52           |
| 5     | 48          | 2,722       | 0            | 100.0%      | 43           |
| 6     | 56          | 2,334       | 0            | 100.0%      | 37           |
| 7     | 64          | 2,042       | 56           | 100.0%      | 32           |
| 8     | 72          | 1,816       | 16           | 100.0%      | 29           |
| 9     | 136         | 962         | 40           | 100.0%      | 16           |
| 10    | 200         | 654         | 112          | 99.9%       | 11           |
| 11    | 264         | 495         | 256          | 99.8%       | 8            |
| 12    | 392         | 334         | 24           | 100.0%      | 6            |
| 13    | 520         | 251         | 448          | 99.7%       | 4            |
| 14    | 648         | 202         | 72           | 99.9%       | 4            |
| 15    | 776         | 168         | 608          | 99.5%       | 3            |
| 16    | 904         | 144         | 800          | 99.4%       | 3            |
| 17    | 1,032       | 126         | 952          | 99.3%       | 2            |
| 18    | 3,080       | 42          | 1,632        | 98.8%       | 1            |
| 19    | 5,128       | 25          | 2,792        | 97.9%       | 1            |
| 20    | 7,176       | 18          | 1,824        | 98.6%       | 1            |
| 21    | 9,224       | 14          | 1,856        | 98.6%       | 1            |
| 22    | 13,320      | 9           | 11,112       | 91.5%       | 1            |
| 23    | 17,416      | 7           | 9,080        | 93.1%       | 1            |
| 24    | 21,512      | 6           | 1,920        | 98.5%       | 1            |
| 25    | 25,608      | 5           | 2,952        | 97.7%       | 1            |
| 26    | 29,704      | 4           | 12,176       | 90.7%       | 1            |
| 27    | 33,800      | 3           | 29,592       | 77.4%       | 1            |
| 28    | 37,896      | 3           | 17,304       | 86.8%       | 1            |
| 29    | 41,992      | 3           | 5,016        | 96.2%       | 1            |
| 30    | 46,088      | 2           | 38,816       | 70.4%       | 1            |
| 31    | 50,184      | 2           | 30,624       | 76.6%       | 1            |

## Summary Statistics

- **Perfect fit classes**: 3 out of 32 (9.4%)
  - Classes 0, 5, and 6 have zero waste
- **Best utilization**: 100.0% (classes 0-9, 12)
- **Worst utilization**: 70.4% (class 30, 46,088 bytes)
- **Average utilization**: 95.5%
- **Utilization range**: 70.4% - 100.0%

## Key Observations

1. **Small objects (≤136 bytes)** achieve perfect or near-perfect utilization
2. **Medium objects (200-1,032 bytes)** maintain >99% utilization
3. **Large objects (>3,000 bytes)** show significant degradation in utilization
4. **Critical threshold** appears around 30,000 bytes where utilization drops below 90%
5. **Bitmap overhead** scales inversely with object size (more objects = more bitmap words needed)

## Potential Optimizations

Based on the data, classes with <90% utilization could benefit from:
- Different segment sizes for large objects
- Alternative allocation strategies for objects >30KB
- Dynamic segment sizing based on object size class

Classes with poor utilization:
- Class 22 (13,320 bytes): 91.5% utilization
- Class 26 (29,704 bytes): 90.7% utilization  
- Class 27 (33,800 bytes): 77.4% utilization
- Class 30 (46,088 bytes): 70.4% utilization