# Bank suite — status and portability notes

Saved exactly as pasted on 2026-09-25. **Not compiled or run**: GnuCOBOL
(`cobc`) is not installed on this machine, so none of the "sample output" or
"9 passed" claims from the source have been verified.

Known problems, taken from the source's own notes plus a read-through:

- `BANKIDX.cob` `seed-file` uses `perform load-record "literal" ...`, which is
  not valid COBOL, and `load-record` is a placeholder no-op. The source's
  portable replacement is below. The file also declares `ws-seek-prefix` in a
  second `working-storage section` after the procedure division.
- `BANKTEST.cob` passes arguments to `perform` (`perform assert-balance 750.00`,
  `assert-balance using ...`); `PERFORM` takes no arguments. The source's
  portable assertion form is below.
- `BANKOO.cob` `withdraw` uses `else if` with a single `end-if`.
- `accounts.dat` lost its fixed-width spacing in the paste: the record layout
  is 8 + 20 + 11 + 1 columns, and the names here are not padded to 20.

## Portable `seed-file` for BANKIDX (from the source)

```cobol
seed-file.
open output account-file
initialize fd-account-record
move 00000001 to fd-account-number
move "Alice Smith" to fd-account-name
move 0012500.50 to fd-account-balance
move "A" to fd-account-status
perform seed-write
move 00000002 to fd-account-number
move "Bob Jones" to fd-account-name
move 0000320.00 to fd-account-balance
move "A" to fd-account-status
perform seed-write
move 00000003 to fd-account-number
move "Carol White" to fd-account-name
move 0007800.75 to fd-account-balance
move "F" to fd-account-status
perform seed-write
move 00000004 to fd-account-number
move "Dave Brown" to fd-account-name
move -000950.25 to fd-account-balance
move "A" to fd-account-status
perform seed-write
close account-file
display "OK: seeded 4 accounts"
.
```

## Portable assertions for BANKTEST (from the source)

```cobol
*> ---- shared assertion operands (add to working-storage) --------------
01 au-expected pic s9(9)v99.
01 au-actual pic s9(9)v99.
01 au-boolean pic x(1) value "N".
88 au-true value "Y".
01 au-test-name pic x(50).
*> ---- portable assertion calls ----------------------------------------
*> assert-balance 750.00
move 750.00 to au-expected
perform assert-balance
*> assert-equal-num for COMPUTE test
move 100.00 to au-expected
move ws-actual to au-actual
move "COMPUTE 50+50 equals 100" to au-test-name
perform assert-equal-num
*> assert-true-when
move "A" to account-status
if account-active set au-true else set au-false end-if
move "account-active is true for 'A'" to au-test-name
perform assert-boolean-true
.
assert-balance.
if account-balance = au-expected
add 1 to cu-passed
display " [PASS] balance = " au-expected
else
add 1 to cu-failed
move account-balance to cu-num-edited
move au-expected to cu-num-edited-2
display " [FAIL] balance expected " cu-num-edited-2
" got " cu-num-edited
end-if
.
assert-equal-num.
if au-expected = au-actual
add 1 to cu-passed
display " [PASS] " au-test-name
else
add 1 to cu-failed
move au-expected to cu-num-edited
move au-actual to cu-num-edited-2
display " [FAIL] " au-test-name
" expected " cu-num-edited " got " cu-num-edited-2
end-if
.
assert-boolean-true.
if au-true
add 1 to cu-passed
display " [PASS] " au-test-name
else
add 1 to cu-failed
display " [FAIL] " au-test-name
end-if
.
assert-boolean-false.
if not au-true
add 1 to cu-passed
display " [PASS] " au-test-name
else
add 1 to cu-failed
display " [FAIL] " au-test-name
end-if
.
```
