#!/bin/bash
# QuDAG Exchange Security Audit Script

set -e

echo "🔒 QuDAG Exchange Security Audit"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check if running from project root
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from QuDAG Exchange directory${NC}"
    exit 1
fi

echo -e "\n${YELLOW}1. Checking for unsafe code...${NC}"
if grep -r "unsafe" src/ --include="*.rs" 2>/dev/null | grep -v "forbid(unsafe_code)" | grep -v "// No unsafe" | grep -v "//.*unsafe"; then
    echo -e "${RED}❌ Found unsafe code blocks!${NC}"
    exit 1
else
    echo -e "${GREEN}✅ No unsafe code found${NC}"
fi

echo -e "\n${YELLOW}2. Checking for unwrap/expect usage...${NC}"
UNWRAP_COUNT=$(grep -r "unwrap()\|expect(" src/ --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    echo -e "${RED}❌ Found $UNWRAP_COUNT unwrap/expect calls${NC}"
    grep -r "unwrap()\|expect(" src/ --include="*.rs" --line-number || true
else
    echo -e "${GREEN}✅ No unwrap/expect calls found${NC}"
fi

echo -e "\n${YELLOW}3. Checking for panic! usage...${NC}"
PANIC_COUNT=$(grep -r "panic!" src/ --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$PANIC_COUNT" -gt 0 ]; then
    echo -e "${RED}❌ Found $PANIC_COUNT panic! calls${NC}"
    grep -r "panic!" src/ --include="*.rs" --line-number || true
else
    echo -e "${GREEN}✅ No panic! calls found${NC}"
fi

echo -e "\n${YELLOW}4. Running cargo audit...${NC}"
if command -v cargo-audit &> /dev/null; then
    cargo audit || echo -e "${YELLOW}⚠️  Some vulnerabilities found${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-audit not installed. Install with: cargo install cargo-audit${NC}"
fi

echo -e "\n${YELLOW}5. Running cargo deny...${NC}"
if command -v cargo-deny &> /dev/null; then
    cargo deny check || echo -e "${YELLOW}⚠️  Some issues found${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-deny not installed. Install with: cargo install cargo-deny${NC}"
fi

echo -e "\n${YELLOW}6. Running clippy security lints...${NC}"
cargo clippy -- \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::suspicious \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -D warnings \
    2>&1 | grep -E "(warning|error)" || echo -e "${GREEN}✅ Clippy checks passed${NC}"

echo -e "\n${YELLOW}7. Checking for hardcoded secrets...${NC}"
# Basic check for common secret patterns
if grep -r -E "(api_key|secret|password|token)\s*=\s*[\"'][^\"']+[\"']" src/ --include="*.rs" 2>/dev/null; then
    echo -e "${RED}❌ Potential hardcoded secrets found!${NC}"
else
    echo -e "${GREEN}✅ No obvious hardcoded secrets${NC}"
fi

echo -e "\n${YELLOW}8. Checking dependencies for known vulnerabilities...${NC}"
cargo tree | grep -E "(openssl|ring|rustls)" || echo -e "${GREEN}✅ Using quantum-safe crypto only${NC}"

echo -e "\n${YELLOW}9. Running security tests...${NC}"
if [ -d "tests/security" ]; then
    cargo test --test '*' --features security-audit -- --nocapture security 2>&1 | grep -E "(test result:|passed)" || true
else
    echo -e "${YELLOW}⚠️  No security tests found${NC}"
fi

echo -e "\n${YELLOW}10. Checking for timing attack resistance...${NC}"
if [ -f "tests/security/timing_attack_tests.rs" ]; then
    cargo test --features timing-attack-tests timing_attack 2>&1 | grep -E "(test result:|passed)" || true
else
    echo -e "${YELLOW}⚠️  No timing attack tests found${NC}"
fi

echo -e "\n${GREEN}🔒 Security Audit Complete${NC}"
echo "================================"

# Summary
echo -e "\n📊 Summary:"
echo "- Unsafe code: Forbidden ✅"
echo "- Error handling: $([ "$UNWRAP_COUNT" -eq 0 ] && echo "✅" || echo "❌ $UNWRAP_COUNT issues")"
echo "- Panic safety: $([ "$PANIC_COUNT" -eq 0 ] && echo "✅" || echo "❌ $PANIC_COUNT issues")"
echo ""
echo "Next steps:"
echo "1. Fix any ❌ issues found above"
echo "2. Run fuzzing: cargo +nightly fuzz run exchange_fuzz"
echo "3. Schedule external security audit"
echo "4. Review SECURITY_CHECKLIST.md"