#!/bin/bash
# Run documentation tests (doctests)

set -e

echo "📚 Running Documentation Tests..."
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Run doctests for each crate
CRATES=(
    "ferric-core"
    "ferric-forms"
    "ferric-http"
    "ferric-idb"
    "ferric-ssr"
    "ferric-markdown"
)

TOTAL_PASSED=0
TOTAL_FAILED=0

for crate in "${CRATES[@]}"; do
    echo -e "${YELLOW}Testing docs in $crate...${NC}"

    if cargo test --doc -p "$crate" 2>&1 | tee /tmp/doctest_$crate.log; then
        # Count passed tests
        PASSED=$(grep -o "test result: ok" /tmp/doctest_$crate.log | wc -l)
        if [ $PASSED -gt 0 ]; then
            echo -e "${GREEN}✓ $crate: $PASSED doctests passed${NC}"
            TOTAL_PASSED=$((TOTAL_PASSED + PASSED))
        fi
    else
        echo -e "${RED}✗ $crate: doctests failed${NC}"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
    fi
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Documentation Test Summary:"
echo "  Total passed: $TOTAL_PASSED"
echo "  Total failed: $TOTAL_FAILED"
echo ""

if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All documentation tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some documentation tests failed${NC}"
    exit 1
fi

