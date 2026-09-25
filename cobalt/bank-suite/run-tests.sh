#!/usr/bin/env bash
# =====================================================================
# COBOL 2024 SUITE — build + automated test runner
# Requires: GnuCOBOL 3.2+ (cobc --version)
# =====================================================================
set -u
PASS=0; FAIL=0; SKIP=0
log() { printf '%s\n' "$*"; }
ok() { PASS=$((PASS+1)); log " [PASS] $*"; }
bad() { FAIL=$((FAIL+1)); log " [FAIL] $*"; }
skip() { SKIP=$((SKIP+1)); log " [SKIP] $*"; }
check() { # $1=desc $2=expected $3=actual
if [ "$2" = "$3" ]; then ok "$1"
else bad "$1 (expected=[$2] got=[$3])"; fi
}
log "== build =="
cobc -x -std=ibm -free -Wall -o bank2024 BANK2024.cob || { log "build failed"; exit 1; }
cobc -x -std=ibm -free -Wall -o bankidx BANKIDX.cob || { log "build failed"; exit 1; }
cobc -x -std=ibm -free -Wall -o bankjson BANKJSON.cob || { log "build failed"; exit 1; }
cobc -x -std=ibm -free -Wall -o bankoo BANKOO.cob || { log "build failed"; exit 1; }
log "== 1. report program =="
OUT=$(./bank2024)
check "exit code" 0 "$?"
check "record count" " Total records : 00005" "$(echo "$OUT" | grep 'Total records')"
log "== 2. indexed file CRUD =="
rm -f bankidx.dat
./bankidx CREATE || bad "seed"
OUT=$(./bankidx READ 00000001)
echo "$OUT" | grep -q "FOUND" && ok "indexed read" || bad "indexed read"
./bankidx UPDATE 00000002
OUT=$(./bankidx READ 00000002)
echo "$OUT" | grep -q "420.00" && ok "rewrite persisted" || bad "rewrite persisted"
./bankidx DELETE 00000004
OUT=$(./bankidx READ 00000004)
echo "$OUT" | grep -q "NOT FOUND" && ok "delete persisted" || bad "delete persisted"
LINES=$(./bankidx LIST | wc -l)
check "list count after delete" 3 "$LINES"
log "== 3. JSON round-trip =="
OUT=$(./bankjson)
echo "$OUT" | grep -q "VERIFIED" && ok "JSON round-trip" || bad "JSON round-trip"
log "== 4. OO COBOL =="
OUT=$(./bankoo)
echo "$OUT" | grep -q "BLOCKED: would breach" && ok "override enforced" || bad "override enforced"
echo "$OUT" | grep -q "OO COBOL DONE" && ok "OO lifecycle" || bad "OO lifecycle"
log "== 5. pure COBOL unit tests =="
cobc -x -std=ibm -free -Wall -o banktest BANKTEST.cob || { log "banktest build failed"; exit 1; }
OUT=$(./banktest)
echo "$OUT" | grep -q "ALL TESTS PASSED" && ok "COBOL unit suite" || bad "COBOL unit suite"
check "unit exit code" 0 "$?"
log ""
log "== RESULTS: $PASS passed, $FAIL failed, $SKIP skipped =="
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
