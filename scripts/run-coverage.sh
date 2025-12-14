#!/bin/bash
# Run code coverage analysis

set -e

echo "🔍 Running Code Coverage Analysis..."
echo ""

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --locked
fi

# Clean previous coverage data
rm -rf coverage/
mkdir -p coverage/

echo "📊 Running tests with coverage instrumentation..."
echo "This may take several minutes..."
echo ""

# Run tarpaulin
cargo tarpaulin \
    --lib \
    --timeout 300 \
    --out Xml \
    --out Html \
    --out Lcov \
    --output-dir coverage \
    --exclude-files 'examples/*' \
    --exclude-files 'benchmarks/*' \
    --exclude-files 'web/*' \
    --exclude-files 'ferric-cli/*' \
    --exclude-files '**/tests/*' \
    -- --test-threads=1

echo ""
echo "✅ Coverage analysis complete!"
echo ""

# Parse coverage percentage from XML
if [ -f coverage/cobertura.xml ]; then
    COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' coverage/cobertura.xml | head -1)
    PERCENT=$(echo "$COVERAGE * 100" | bc)
    echo "📈 Overall Coverage: ${PERCENT}%"
    echo ""

    # Check if we met the 90% threshold
    if (( $(echo "$COVERAGE >= 0.90" | bc -l) )); then
        echo "🎉 Coverage goal of 90% MET!"
    else
        echo "⚠️  Coverage is below 90% threshold"
        echo "   Current: ${PERCENT}%"
        echo "   Target: 90%"
    fi
fi

echo ""
echo "📁 Coverage report generated at: ./coverage/index.html"
echo "   Open with: firefox ./coverage/index.html"
echo ""
echo "📄 Files:"
ls -lh coverage/ | grep -v "^total" | awk '{print "   " $9 " (" $5 ")"}'

