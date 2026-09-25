*> =====================================================================
*> PROGRAM : BANK2024
*> AUTHOR : GLM / Generated 2024-09
*> DIALECT : GnuCOBOL 3.2+ (COBOL 202x, free-format)
*> PURPOSE : Modern COBOL showcase — accounts, validation, reporting
*> COMPILE : cobc -x -std=ibm -free -Wall BANK2024.cob
*> =====================================================================
identification division.
program-id. BANK2024.
environment division.
configuration section.
repository.
function all intrinsic. *> all FUNCTION-xxx usable
input-output section.
file-control.
select account-file assign to "accounts.dat"
organization is line sequential
file status is ws-file-status.
data division.
file section.
fd account-file.
01 fd-account-record.
05 fd-account-number pic 9(8).
05 fd-account-name pic x(20).
05 fd-account-balance pic s9(9)v99.
05 fd-account-status pic x(1).
88 fd-active value "A".
88 fd-frozen value "F".
working-storage section.
01 ws-file-status pic xx value spaces.
01 ws-summary.
05 ws-total-balance pic s9(12)v99 value zero.
05 ws-record-count pic 9(5) value zero.
05 ws-active-count pic 9(5) value zero.
05 ws-max-balance pic s9(9)v99 value zero.
01 ws-working-account.
05 ws-account-number pic 9(8).
05 ws-account-name pic x(20).
05 ws-account-balance pic s9(9)v99.
05 ws-account-status pic x(1).
88 ws-active value "A".
88 ws-valid-status value "A" "F" "C".
01 ws-flags.
05 ws-eof-flag pic a value "N".
88 end-of-file value "Y".
88 more-records value "N".
05 ws-error-count pic 9(3) value zero.
01 ws-header pic x(44) value
" ACCOUNT NAME BALANCE".
01 ws-line.
05 filler pic x value space.
05 ws-display-number pic 9(8).
05 filler pic x value space.
05 ws-display-name pic x(20).
05 filler pic x value space.
05 ws-display-balance pic z(9)9.99-.
05 filler pic x value space.
05 ws-display-status pic x(9).
01 ws-report-line pic x(60).
*> ---------------------------------------------------------------------
procedure division.
main-paragraph.
display "==========================================" upon console
display " MODERN COBOL 2024 - ACCOUNT REPORT " upon console
display "==========================================" upon console
perform initialize-system
perform open-input
perform read-and-process-until-eof
perform close-input
perform display-summary
perform terminate-system
goback
.
*> ---------------------------------------------------------------------
initialize-system.
move zero to ws-total-balance
move zero to ws-record-count
move zero to ws-active-count
move zero to ws-max-balance
move zero to ws-error-count
set more-records to true
.
*> ---------------------------------------------------------------------
open-input.
open input account-file
evaluate true
when ws-file-status = "00"
continue
when other
display "ERROR: cannot open file, status=" ws-file-status
move 1 to return-code
stop run
end-evaluate
.
*> ---------------------------------------------------------------------
read-and-process-until-eof.
perform until end-of-file
read account-file
at end
set end-of-file to true
not at end
perform move-record-to-working
perform validate-record
if ws-error-count = zero
perform process-account
end-if
end-read
end-perform
.
*> ---------------------------------------------------------------------
move-record-to-working.
move fd-account-number to ws-account-number
move fd-account-name to ws-account-name
move fd-account-balance to ws-account-balance
move fd-account-status to ws-account-status
.
*> ---------------------------------------------------------------------
validate-record.
if ws-account-number = zero
display "WARNING: invalid account number in record "
ws-record-count + 1
add 1 to ws-error-count
exit paragraph
end-if
if ws-account-balance < -10000
display "WARNING: overdrawn beyond limit: "
ws-account-number
add 1 to ws-error-count
exit paragraph
end-if
if not ws-valid-status
display "WARNING: unknown status code: " ws-account-status
add 1 to ws-error-count
end-if
.
*> ---------------------------------------------------------------------
process-account.
add 1 to ws-record-count
add ws-account-balance to ws-total-balance
if ws-active
add 1 to ws-active-count
end-if
if ws-account-balance > ws-max-balance
move ws-account-balance to ws-max-balance
end-if
move ws-account-number to ws-display-number
move ws-account-name to ws-display-name
move ws-account-balance to ws-display-balance
evaluate true
when ws-active move "ACTIVE" to ws-display-status
when fd-frozen move "FROZEN" to ws-display-status
when other move "CLOSED" to ws-display-status
end-evaluate
string ws-line delimited by size into ws-report-line
display ws-line
.
*> ---------------------------------------------------------------------
close-input.
close account-file
.
*> ---------------------------------------------------------------------
display-summary.
display "=========================================="
display " Total records : " ws-record-count
display " Active accounts: " ws-active-count
display " Total balance : " ws-total-balance
display " Max balance : " ws-max-balance
display " Skipped errors : " ws-error-count
*> Modern intrinsic function usage
display " Mean balance : "
function mean(ws-total-balance ws-record-count)
display "=========================================="
.
*> ---------------------------------------------------------------------
terminate-system.
evaluate true
when ws-error-count = zero
move 0 to return-code
display "STATUS: SUCCESS"
when other
move 4 to return-code
display "STATUS: COMPLETED WITH WARNINGS"
end-evaluate
.
end program BANK2024.
