# Performance Optimizations

This document details the performance optimizations applied to the Rusty log analyzer.

## Key Optimizations

### 1. **Pre-compiled Static Regex**
- **Before**: Regex was compiled on every file processing
- **After**: Using `once_cell::sync::Lazy` to compile regex once at startup
- **Impact**: Eliminates redundant regex compilation overhead

### 2. **Larger Buffer Size**
- **Before**: Default BufReader buffer (8KB)
- **After**: 64KB buffer size
- **Impact**: Reduces system calls for file I/O, better throughput for large files

### 3. **Local Batching**
- **Before**: Every IP occurrence updated the global DashMap immediately
- **After**: Accumulate counts locally, batch update every 10,000 lines
- **Impact**: Dramatically reduces lock contention on the shared DashMap

### 4. **Early Filtering**
- **Before**: Every line goes through regex matching
- **After**: Quick check if line starts with a digit before regex
- **Impact**: Skips regex processing for non-IP lines (errors, warnings, etc.)

### 5. **Pre-allocated Capacity**
- **Before**: DashMap and HashMap grow dynamically
- **After**: Pre-allocate with reasonable capacity (10K for global, 1K for local)
- **Impact**: Reduces memory reallocation and rehashing

### 6. **Better Data Types**
- **Before**: Using `i32` for counts
- **After**: Using `u64` for counts and line tracking
- **Impact**: Handles larger files, prevents overflow, atomic operations for stats

### 7. **Build Optimizations**
- `opt-level = 3`: Maximum optimization
- `lto = "fat"`: Link-time optimization across all crates
- `codegen-units = 1`: Better optimization at cost of compile time
- `strip = true`: Smaller binary size

### 8. **Enhanced Output**
- Added timing information
- Shows total lines processed
- Sorts IPs by count (most frequent first)
- Limits output to top 20 IPs for better readability

## Performance Characteristics

These optimizations make the analyzer particularly effective for:
- Very large log files (GB+ size)
- Files with high IP diversity
- Mixed content logs (with non-IP lines)
- Multi-file concurrent processing

## Benchmark Suggestions

To measure the impact, consider benchmarking with:
```bash
# Generate a large test file
for i in {1..1000000}; do
    echo "$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256)) - - [timestamp] \"GET /path HTTP/1.0\" 200 1234"
done > large_test.log

# Time the analysis
time ./target/release/rusty large_test.log
```

## Future Optimization Ideas

1. **Memory-mapped files**: For extremely large files
2. **SIMD operations**: For IP parsing (using `packed_simd`)
3. **Custom IP parser**: Replace regex with hand-written parser
4. **Probabilistic counting**: HyperLogLog for approximate unique IP counts
5. **Streaming aggregation**: Process files in chunks without loading full lines
