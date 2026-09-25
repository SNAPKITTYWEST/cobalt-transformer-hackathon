*> =====================================================================
*> PROGRAM : BANKJSON
*> PURPOSE : JSON GENERATE + JSON PARSE on account data
*> COMPILE : cobc -x -std=ibm -free -Wall BANKJSON.cob
*> NOTE : JSON PARSE requires GnuCOBOL 3.2+ with JSON support
*> =====================================================================
identification division.
program-id. BANKJSON.
environment division.
data division.
working-storage section.
01 ws-json pic x(512).
01 ws-json-len binary-long.
01 ws-account.
05 ws-account-number pic 9(8).
05 ws-account-name pic x(20).
05 ws-account-balance pic s9(9)v99.
05 ws-account-status pic x(1).
88 ws-active value "A".
01 ws-parsed.
05 ws-p-number pic 9(8).
05 ws-p-name pic x(20).
05 ws-p-balance pic s9(9)v99.
05 ws-p-status pic x(1).
01 ws-exit-code binary-long value zero.
procedure division.
main.
*> ---------- 1. GENERATE JSON from a group item ----------------------
move 00000042 to ws-account-number
move "Grace Hopper" to ws-account-name
move 0009999.99 to ws-account-balance
move "A" to ws-account-status
json generate ws-json
from ws-account
count in ws-json-len
on exception
display "ERROR: JSON GENERATE failed"
move 8 to return-code
goback
end-json
display "GENERATED JSON (" ws-json-len " bytes):"
display ws-json(1:ws-json-len)
*> ---------- 2. PARSE JSON back into a group item --------------------
move spaces to ws-parsed
json parse ws-json(1:ws-json-len)
into ws-parsed
on exception
display "ERROR: JSON PARSE failed"
move 8 to return-code
goback
not on exception
display "PARSED OK:"
display " number : " ws-p-number
display " name : " ws-p-name
display " balance: " ws-p-balance
display " status : " ws-p-status
end-json
*> ---------- 3. Round-trip verification ------------------------------
if ws-p-number = ws-account-number
and ws-p-name = ws-account-name
and ws-p-balance = ws-account-balance
and ws-p-status = ws-account-status
display "ROUND-TRIP: VERIFIED ✓"
else
display "ROUND-TRIP: MISMATCH ✗"
move 4 to return-code
end-if
goback
.
end program BANKJSON.
