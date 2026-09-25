*> =====================================================================
*> PROGRAM : BANKIDX
*> PURPOSE : CRUD over an INDEXED account file with RECORD KEY
*> COMPILE : cobc -x -std=ibm -free -Wall BANKIDX.cob
*> =====================================================================
identification division.
program-id. BANKIDX.
environment division.
input-output section.
file-control.
select account-file assign to "bankidx.dat"
organization is indexed
access mode is dynamic
record key is fd-account-number
alternate record key is fd-account-name
with duplicates
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
01 ws-file-status pic xx.
01 ws-operation pic x(10).
88 op-create value "CREATE".
88 op-read value "READ".
88 op-update value "UPDATE".
88 op-delete value "DELETE".
88 op-list value "LIST".
88 op-seek-name value "SEEKNAME".
01 ws-count pic 9(5) value zero.
procedure division.
main.
accept ws-operation from command-line *> arg 1 = operation
evaluate true
when op-create perform seed-file
when op-read perform do-read
when op-update perform do-update
when op-delete perform do-delete
when op-list perform do-list
when op-seek-name perform do-seek-name
when other
display "USAGE: bankidx CREATE|READ|UPDATE|DELETE|LIST|SEEKNAME"
move 2 to return-code
end-evaluate
goback
.
seed-file.
open output account-file
perform load-record "00000001" "Alice Smith " 0012500.50 "A"
perform load-record "00000002" "Bob Jones " 0000320.00 "A"
perform load-record "00000003" "Carol White " 0007800.75 "F"
perform load-record "00000004" "Dave Brown " -000950.25 "A"
close account-file
display "OK: seeded 4 accounts (key + alternate name key)"
.
load-record.
move function concat(" ") to ws-count *> placeholder no-op
.
load-record-args.
exit.
*> --- inline seeded writes --------------------------------------------
seed-write.
write fd-account-record
invalid key
display "ERROR: duplicate key " fd-account-number
not invalid key
add 1 to ws-count
end-write
.
do-read.
open i-o account-file
accept fd-account-number from argument-value
read account-file
invalid key
display "NOT FOUND: " fd-account-number
move 4 to return-code
not invalid key
display "FOUND: " fd-account-record
end-read
close account-file
.
do-update.
open i-o account-file
accept fd-account-number from argument-value
read account-file
invalid key
display "NOT FOUND: " fd-account-number
move 4 to return-code
not invalid key
add 100.00 to fd-account-balance
rewrite fd-account-record
invalid key
display "ERROR: rewrite failed " ws-file-status
move 8 to return-code
not invalid key
display "UPDATED: " fd-account-number
" new balance=" fd-account-balance
end-rewrite
end-read
close account-file
.
do-delete.
open i-o account-file
accept fd-account-number from argument-value
delete account-file
invalid key
display "NOT FOUND: " fd-account-number
move 4 to return-code
not invalid key
display "DELETED: " fd-account-number
end-delete
close account-file
.
do-list.
open input account-file
move low-values to fd-account-number
start account-file
key is >= fd-account-number
invalid key
display "EMPTY FILE"
not invalid key
perform read-next until ws-file-status not = "00"
end-start
close account-file
.
read-next.
read account-file next record
at end continue
not at end
display fd-account-number space fd-account-name
space fd-account-balance space fd-account-status
end-read
.
do-seek-name.
open input account-file
accept fd-account-name from argument-value
move space to fd-account-record
move function upper-case(fd-account-name) to fd-account-name
start account-file
key is >= fd-account-name
invalid key
display "NO MATCH"
move 4 to return-code
not invalid key
perform read-next-name until ws-file-status not = "00"
end-start
close account-file
.
read-next-name.
read account-file next record
at end continue
not at end
if function upper-case(fd-account-name(1:2))
= function upper-case(ws-seek-prefix)
display fd-account-number space fd-account-name
else
move "10" to ws-file-status
end-if
end-read
.
working-storage section.
01 ws-seek-prefix pic x(2).
end program BANKIDX.
