*> =====================================================================
*> PROGRAM : BANKTEST
*> PURPOSE : Unit tests for the account logic (self-contained)
*> COMPILE : cobc -x -std=ibm -free -Wall BANKTEST.cob
*> =====================================================================
identification division.
program-id. BANKTEST.
environment division.
data division.
working-storage section.
copy "COBUNIT".
*> ---- unit under test: account rules --------------------------------
01 account-balance pic s9(9)v99 value zero.
01 withdrawal-amount pic s9(9)v99 value zero.
01 transaction-status pic x(8) value spaces.
88 status-approved value "APPROVED".
88 status-declined value "DECLINED".
88 status-frozen value "FROZEN".
01 account-status pic x(1) value "A".
88 account-active value "A".
88 account-frozen value "F".
01 ws-expected pic s9(9)v99.
01 ws-actual pic s9(9)v99.
procedure division.
main.
perform suite-withdrawal-rules
perform suite-status-rules
perform suite-round-trip
perform cu-report
evaluate true
when cu-failed = zero move 0 to return-code
when other move 1 to return-code
end-evaluate
goback
.
*> =====================================================================
*> SUITE 1 — withdrawal authorization rules
*> =====================================================================
suite-withdrawal-rules.
move "WITHDRAWAL RULES" to cu-current
display "SUITE: " cu-current
*> TEST 1 — sufficient funds
perform test-begin
move 1000.00 to account-balance
move 250.00 to withdrawal-amount
perform apply-withdrawal
perform assert-approved
perform assert-balance 750.00
*> TEST 2 — exact funds
perform test-begin
move 100.00 to account-balance
move 100.00 to withdrawal-amount
perform apply-withdrawal
perform assert-approved
perform assert-balance 0.00
*> TEST 3 — insufficient funds
perform test-begin
move 50.00 to account-balance
move 100.00 to withdrawal-amount
perform apply-withdrawal
perform assert-declined
perform assert-balance 50.00
*> TEST 4 — frozen account
perform test-begin
move 500.00 to account-balance
move 10.00 to withdrawal-amount
set account-frozen to true
perform apply-withdrawal
perform assert-frozen-status
perform assert-balance 500.00
set account-active to true
.
*> ---- unit under test (paragraph-level) ------------------------------
apply-withdrawal.
evaluate true
when account-frozen
move "FROZEN" to transaction-status
when account-balance >= withdrawal-amount
subtract withdrawal-amount from account-balance
move "APPROVED" to transaction-status
when other
move "DECLINED" to transaction-status
end-evaluate
.
*> =====================================================================
*> SUITE 2 — status validation rules
*> =====================================================================
suite-status-rules.
move "STATUS RULES" to cu-current
display "SUITE: " cu-current
move "A" to account-status
perform assert-true-when
account-active
"account-active is true for 'A'"
move "F" to account-status
perform assert-true-when
account-frozen
"account-frozen is true for 'F'"
move "Z" to account-status
perform assert-false-when
account-active
"account-active is false for 'Z'"
.
*> =====================================================================
*> SUITE 3 — numeric round-trip
*> =====================================================================
suite-round-trip.
move "NUMERIC ROUND-TRIP" to cu-current
display "SUITE: " cu-current
perform test-begin
move 12500.50 to ws-expected
move ws-expected to ws-actual
add 0.00 to ws-actual
perform assert-equal-num ws-expected ws-actual
"MOVE/ADD round-trip preserves value"
perform test-begin
move 100.00 to ws-expected
compute ws-actual = 50.00 + 50.00
perform assert-equal-num ws-expected ws-actual
"COMPUTE 50+50 equals 100"
.
*> =====================================================================
*> ASSERTION LIBRARY (COBUNIT paragraphs)
*> =====================================================================
test-begin.
add 1 to cu-total
move spaces to cu-msg
.
assert-balance using expected-balance.
if account-balance = expected-balance
add 1 to cu-passed
display " [PASS] balance = " expected-balance
else
add 1 to cu-failed
move account-balance to cu-num-edited
move expected-balance to cu-num-edited-2
display " [FAIL] balance expected " cu-num-edited-2
" got " cu-num-edited
end-if
.
assert-approved.
if status-approved
add 1 to cu-passed
display " [PASS] status = APPROVED"
else
add 1 to cu-failed
display " [FAIL] expected APPROVED got " transaction-status
end-if
.
assert-declined.
if status-declined
add 1 to cu-passed
display " [PASS] status = DECLINED"
else
add 1 to cu-failed
display " [FAIL] expected DECLINED got " transaction-status
end-if
.
assert-frozen-status.
if status-frozen
add 1 to cu-passed
display " [PASS] status = FROZEN"
else
add 1 to cu-failed
display " [FAIL] expected FROZEN got " transaction-status
end-if
.
assert-true-when using cond-value test-name.
if cond-value
add 1 to cu-passed
display " [PASS] " test-name
else
add 1 to cu-failed
display " [FAIL] " test-name
end-if
.
assert-false-when using cond-value test-name.
if not cond-value
add 1 to cu-passed
display " [PASS] " test-name
else
add 1 to cu-failed
display " [FAIL] " test-name
end-if
.
assert-equal-num using expected-value actual-value test-name.
if expected-value = actual-value
add 1 to cu-passed
display " [PASS] " test-name
else
add 1 to cu-failed
move expected-value to cu-num-edited
move actual-value to cu-num-edited-2
display " [FAIL] " test-name
" expected " cu-num-edited " got " cu-num-edited-2
end-if
.
cu-report.
display "=========================================="
display " UNIT TEST SUMMARY"
display " Suite : BANKTEST"
display " Total : " cu-total
display " Passed : " cu-passed
display " Failed : " cu-failed
display "=========================================="
evaluate true
when cu-failed = zero
display " RESULT : ALL TESTS PASSED"
when other
display " RESULT : FAILURES DETECTED"
end-evaluate
.
end program BANKTEST.
