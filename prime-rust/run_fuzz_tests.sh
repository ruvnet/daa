#!/bin/bash

# Prime-Rust Comprehensive Fuzz Testing Script

set -e

echo "🚀 Starting Prime-Rust Fuzz Testing Suite"
echo "========================================"

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo "Installing cargo-fuzz..."
    cargo install cargo-fuzz
fi

# Change to fuzz directory
cd fuzz

# List of fuzz targets
FUZZ_TARGETS=(
    "protocol_message_fuzz"
    "gradient_aggregation_fuzz"
    "dht_operations_fuzz"
    "consensus_message_fuzz"
    "serialization_fuzz"
)

# Duration for each fuzz target (in seconds)
FUZZ_DURATION=${FUZZ_DURATION:-60}
MAX_TOTAL_TIME=${MAX_TOTAL_TIME:-300}

echo "Running each fuzz target for ${FUZZ_DURATION} seconds"
echo "Maximum total time: ${MAX_TOTAL_TIME} seconds"
echo ""

# Create directory for artifacts
mkdir -p artifacts
mkdir -p corpus

# Function to run a single fuzz target
run_fuzz_target() {
    local target=$1
    local duration=$2
    
    echo "🔍 Fuzzing target: $target"
    echo "Duration: ${duration}s"
    
    # Create target-specific corpus directory
    mkdir -p corpus/$target
    
    # Run the fuzz target with timeout
    timeout ${duration}s cargo fuzz run $target -- -max_total_time=${duration} \
        -artifact_prefix=artifacts/${target}_ \
        corpus/$target/ || {
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo "✅ Fuzzing completed for $target (timeout reached)"
        elif [ $exit_code -eq 77 ]; then
            echo "🔥 CRASH FOUND in $target! Check artifacts/"
            return 77
        else
            echo "❌ Fuzzing failed for $target with exit code $exit_code"
            return $exit_code
        fi
    }
    
    echo ""
}

# Track overall results
TOTAL_CRASHES=0
FAILED_TARGETS=()

# Run each fuzz target
for target in "${FUZZ_TARGETS[@]}"; do
    if run_fuzz_target "$target" "$FUZZ_DURATION"; then
        echo "✅ $target completed successfully"
    else
        exit_code=$?
        if [ $exit_code -eq 77 ]; then
            echo "🔥 CRASH DETECTED in $target"
            TOTAL_CRASHES=$((TOTAL_CRASHES + 1))
            FAILED_TARGETS+=("$target")
        else
            echo "❌ $target failed"
            FAILED_TARGETS+=("$target")
        fi
    fi
done

echo ""
echo "🏁 Fuzz Testing Summary"
echo "======================"
echo "Total targets: ${#FUZZ_TARGETS[@]}"
echo "Crashes found: $TOTAL_CRASHES"
echo "Failed targets: ${#FAILED_TARGETS[@]}"

if [ ${#FAILED_TARGETS[@]} -gt 0 ]; then
    echo ""
    echo "Failed/Crashed targets:"
    for target in "${FAILED_TARGETS[@]}"; do
        echo "  - $target"
    done
fi

# List any artifacts found
if [ -n "$(ls -A artifacts/ 2>/dev/null)" ]; then
    echo ""
    echo "🔍 Artifacts found:"
    ls -la artifacts/
    echo ""
    echo "To reproduce crashes, run:"
    for artifact in artifacts/*; do
        if [ -f "$artifact" ]; then
            target_name=$(basename "$artifact" | cut -d'_' -f1)
            echo "  cargo fuzz run $target_name $artifact"
        fi
    done
fi

# Coverage information
echo ""
echo "📊 Generating coverage report..."
if command -v cargo-fuzz &> /dev/null; then
    for target in "${FUZZ_TARGETS[@]}"; do
        if [ -d "corpus/$target" ] && [ -n "$(ls -A corpus/$target 2>/dev/null)" ]; then
            echo "Coverage for $target:"
            timeout 30s cargo fuzz coverage $target || echo "  Coverage generation failed or timed out"
        fi
    done
fi

# Minimize corpus
echo ""
echo "🧹 Minimizing corpus..."
for target in "${FUZZ_TARGETS[@]}"; do
    if [ -d "corpus/$target" ] && [ -n "$(ls -A corpus/$target 2>/dev/null)" ]; then
        echo "Minimizing corpus for $target..."
        timeout 60s cargo fuzz cmin $target || echo "  Corpus minimization failed or timed out"
    fi
done

echo ""
echo "🎯 Fuzz testing completed!"

if [ $TOTAL_CRASHES -gt 0 ]; then
    echo "⚠️  CRASHES DETECTED! Please investigate the artifacts."
    exit 1
else
    echo "✅ No crashes detected. Good job!"
    exit 0
fi