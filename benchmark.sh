#!/bin/bash
set -e

# Build release version
cargo build --release

# Create a large test file (approx 480KB)
echo "Generating large test file..."
echo "<root>" > large_test.xml
for i in {1..2000}; do cat tests/data/xml/formatted.xml >> large_test.xml; done
echo "</root>" >> large_test.xml

echo "Running benchmarks with hyperfine..."
hyperfine --warmup 3 --export-markdown benchmark_results.md \
    -n "rxq format" "./target/release/rxq large_test.xml > /dev/null" \
    -n "xq format" "xq large_test.xml > /dev/null" \
    -n "rxq extract" "./target/release/rxq -x //user/first_name large_test.xml > /dev/null" \
    -n "xq extract" "xq -x //user/first_name large_test.xml > /dev/null" \
    -n "rxq json" "./target/release/rxq --json large_test.xml > /dev/null" \
    -n "xq json" "xq --json large_test.xml > /dev/null"

echo "Done. Results saved to benchmark_results.md"
rm large_test.xml
