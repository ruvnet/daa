#!/bin/bash
# QuDAG Exchange Verification Fuzzing Script
# Run all fuzzing targets and collect results

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/verification_results"
CORPUS_DIR="$SCRIPT_DIR/corpus"
ARTIFACTS_DIR="$SCRIPT_DIR/artifacts"

# Create directories
mkdir -p "$RESULTS_DIR" "$CORPUS_DIR" "$ARTIFACTS_DIR"

# Fuzzing configuration
FUZZ_TIME=${FUZZ_TIME:-60}  # Default 60 seconds per target
FUZZ_JOBS=${FUZZ_JOBS:-$(nproc)}  # Use all CPU cores

# List of fuzzing targets
TARGETS=(
    "fuzz_ruv_transactions"
    "fuzz_ledger_consistency"
    "fuzz_consensus_transitions"
    "fuzz_resource_metering"
    "fuzz_wallet_operations"
    "fuzz_zk_proofs"
    "fuzz_serialization"
)

echo "=== QuDAG Exchange Verification Fuzzing ==="
echo "Time per target: $FUZZ_TIME seconds"
echo "Parallel jobs: $FUZZ_JOBS"
echo

# Function to run a single fuzzer
run_fuzzer() {
    local target=$1
    local log_file="$RESULTS_DIR/${target}.log"
    local corpus_dir="$CORPUS_DIR/$target"
    local artifact_dir="$ARTIFACTS_DIR/$target"
    
    mkdir -p "$corpus_dir" "$artifact_dir"
    
    echo "Running $target..."
    
    # Run fuzzer with timeout
    if timeout "${FUZZ_TIME}s" cargo +nightly fuzz run "$target" \
        --jobs "$FUZZ_JOBS" \
        -- \
        -max_len=65536 \
        -len_control=100 \
        -artifact_prefix="$artifact_dir/" \
        "$corpus_dir" \
        > "$log_file" 2>&1; then
        echo "✓ $target completed successfully"
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo "✓ $target timeout (expected)"
        else
            echo "✗ $target failed with exit code $exit_code"
            tail -n 20 "$log_file"
        fi
    fi
    
    # Extract statistics
    if grep -q "stat::" "$log_file"; then
        echo "  Statistics:"
        grep "stat::" "$log_file" | tail -n 1 | sed 's/^/    /'
    fi
    
    # Check for crashes
    local crashes=$(find "$artifact_dir" -name "crash-*" 2>/dev/null | wc -l)
    if [ "$crashes" -gt 0 ]; then
        echo "  ⚠️  Found $crashes crashes in $target"
    fi
    
    echo
}

# Install cargo-fuzz if not present
if ! command -v cargo-fuzz &> /dev/null; then
    echo "Installing cargo-fuzz..."
    cargo install cargo-fuzz
fi

# Run all fuzzers
for target in "${TARGETS[@]}"; do
    run_fuzzer "$target"
done

# Generate summary report
echo "=== Fuzzing Summary ==="
echo "Results saved to: $RESULTS_DIR"
echo

# Count total crashes
total_crashes=$(find "$ARTIFACTS_DIR" -name "crash-*" 2>/dev/null | wc -l)
echo "Total crashes found: $total_crashes"

# List crashes by target
if [ "$total_crashes" -gt 0 ]; then
    echo
    echo "Crashes by target:"
    for target in "${TARGETS[@]}"; do
        crashes=$(find "$ARTIFACTS_DIR/$target" -name "crash-*" 2>/dev/null | wc -l)
        if [ "$crashes" -gt 0 ]; then
            echo "  - $target: $crashes crashes"
        fi
    done
fi

# Generate JSON report for Memory system
cat > "$RESULTS_DIR/verification_report.json" <<EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "fuzz_time_per_target": $FUZZ_TIME,
    "total_crashes": $total_crashes,
    "targets": {
EOF

first=true
for target in "${TARGETS[@]}"; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> "$RESULTS_DIR/verification_report.json"
    fi
    
    crashes=$(find "$ARTIFACTS_DIR/$target" -name "crash-*" 2>/dev/null | wc -l)
    corpus_size=$(find "$CORPUS_DIR/$target" -type f 2>/dev/null | wc -l)
    
    cat >> "$RESULTS_DIR/verification_report.json" <<EOF
        "$target": {
            "crashes": $crashes,
            "corpus_size": $corpus_size,
            "log_file": "${target}.log"
        }
EOF
done

cat >> "$RESULTS_DIR/verification_report.json" <<EOF

    }
}
EOF

echo
echo "Verification report saved to: $RESULTS_DIR/verification_report.json"