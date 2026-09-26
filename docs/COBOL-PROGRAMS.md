# COBOL program reference

What each COBOL program in this repo does, grouped by system. Paths are relative
to the repo root. Where the same file was collected from several places, the
**canonical** copy is listed first. Copies that differ only in line endings are
listed as "also at".

- [1. How the pieces fit together](#1-how-the-pieces-fit-together)
- [2. COBILT libraries (IBM i, REXX-orchestrated)](#2-cobilt-libraries-ibm-i-rexx-orchestrated)
- [3. ACH returns and ledger posting](#3-ach-returns-and-ledger-posting)
- [4. WORM ledger suite (worm-pow-rsa)](#4-worm-ledger-suite-worm-pow-rsa)
- [5. Bridges to Rust and PL/I](#5-bridges-to-rust-and-pli)
- [6. Agent and kernel programs](#6-agent-and-kernel-programs)
- [7. Copybooks (record layouts only)](#7-copybooks-record-layouts-only)
- [8. Cobalt: REQPARSE and the bank suite](#8-cobalt-reqparse-and-the-bank-suite)
- [9. Notes for the transformer](#9-notes-for-the-transformer)

---

## 1. How the pieces fit together

Four patterns repeat across the code:

| Pattern | Programs | Shape |
|---|---|---|
| **REXX → COBOL library → DB2 / Dataworm** | COBILT-VAULT, COBILT-DATAWORM, COBILT-DATAWORM-TREASURY, COBILT-ACH-TREASURY | A REXX orchestrator passes a command string; the COBOL library dispatches it with `EVALUATE`, runs Datalog-style logic (facts, rules, unify, backtrack), and journals every step. |
| **ACH money movement** | COBILT-ACH-TREASURY → ACHRTRN / ACHMATRIX → LEDGWYCB → LEDGER_POST | Origination (batch → validate → NACHA file → submit → settle → reconcile), then returns (state machine to `RETURNED` with NACHA reason codes), then reversal and ledger posting. |
| **Upstream verifies, COBOL enforces order** | worm-pow-rsa suite, SOV-RECORD-GATE | Cryptography (hashes, PoW, signatures) is computed upstream in Ada/SPARK or PL/I; COBOL only checks sequence, hash-link string equality, and seal flags, and never rewrites a ledger. |
| **COBOL as a gate for another runtime** | SNAPKITTY-TRANSACTION-BRIDGE, TRSY-WORM-BRIDGE, VAULT-TREASURY | Input arrives through environment variables, stdin, or a file; COBOL validates it and emits JSON or `key=value` lines for Rust or an agent to consume. |

---

## 2. COBILT libraries (IBM i, REXX-orchestrated)

### COBILT-VAULT — deterministic logic vault
**File:** `sources/C/Users/jessi/Desktop/bobs control repo/COBILT-VAULT.cbl` (658 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/COBILT-VAULT.cbl`

A callable subprogram (`PROCEDURE DIVISION USING LK-REQUEST LK-RESPONSE`) that combines a key/value vault, a small logic engine, and bridge entry points for REXX, RPGLE and COBOL callers. It declares `SECURITY. FAIL-CLOSED` and targets IBM i.

**Interface**
- `LK-REQUEST`: `LK-COMMAND` (32), `LK-KEY` (64), `LK-VALUE` (256), `LK-ACTOR` (64), `LK-ARG1..3` (128 each).
- `LK-RESPONSE`: `LK-STATUS` (16), `LK-CODE` 9(8), `LK-MESSAGE` (256), `LK-HASH` (64), `LK-SEQUENCE` 9(18).
- Vault record (`VAULT-LEDGER`): key, type, state, value, hash, sequence, owner, timestamp, authority.

**Commands** (`DISPATCH-COMMAND`)

| Command | Paragraph | Success code | Reject code |
|---|---|---|---|
| VAULT-OPEN | VAULT-OPEN-OPERATION | 1 | — |
| VAULT-READ | VAULT-READ-OPERATION | 2 | 9998 (read failed) |
| VAULT-WRITE | VAULT-WRITE-OPERATION | 3 (COMMITTED) | 13 (no authority) |
| VAULT-ASSERT | VAULT-ASSERT-OPERATION | 4 | 14 |
| VAULT-QUERY | LOGIC-QUERY | 5 | 15 |
| VAULT-UNIFY | LOGIC-UNIFY | 6 | 16 |
| VAULT-BACKTRACK | LOGIC-BACKTRACK | 7 | 17 (depth > 9999) |
| VAULT-COMMIT | VAULT-COMMIT-OPERATION | 8 | 9997 |
| VAULT-ROLLBACK | VAULT-ROLLBACK-OPERATION | 9 | — |
| BRIDGE-REXX / BRIDGE-RPGLE / BRIDGE-COBOL | PROCESS-*-COMMAND | sets the matching bridge flag | — |
| anything else | COMMAND-REJECT | — | 9999 |

**How it works**
- **Write** requires a non-blank actor (`VALIDATE-AUTHORITY`). It then builds the "hash" by `STRING`ing key + value + authority + previous hash into a 64-byte field and replacing spaces with zeros (`HASH-NORMALIZE`). This is a chaining fingerprint, not a cryptographic hash. `APPEND-VAULT-RECORD` fills the vault record, bumps the sequence, and carries the hash forward as the next record's previous hash.
- **Logic engine:** a fact holds when its predicate and first argument are non-blank. Unification succeeds when both terms are equal, or when one is blank, in which case it takes the other's value. A query tries the fact base first, then the rule base, which pushes a choice point. Backtracking pops choice points up to a maximum depth of 9999.
- **Extra paragraph families**, not reachable from `DISPATCH-COMMAND` but available for `PERFORM` inside the program:
  - predicates: `PRED-EXISTS`, `-EQUAL`, `-NOT-EQUAL`, `-PRESENT`, `-AUTHORIZED`
  - rule combinators: `RULE-AND`, `-OR`, `-NOT`, `-CHAIN`
  - a transaction gate: `TRANSACTION-BEGIN`, `-VALIDATE`, `-COMMIT`, `-ROLLBACK`
  - RPGLE and REXX contract wrappers
  - DB2 read, insert, commit and rollback semantics
  - integrity checks: `VERIFY-SEQUENCE`, `-HASH`, `-AUTHORITY`, `-ENTRY`
- **No file I/O:** `READ-VAULT-RECORD` clears the fields rather than reading a file, and the record is built in memory without a `WRITE`. Persistence belongs to the DB2 layer the header names.

### COBILT-DATAWORM — Datalog storage engine
**File:** `sources/C/Users/jessi/Desktop/bobs control repo/COBILT-DATAWORM.cbl` (476 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/COBILT-DATAWORM.cbl`

"Dataworm replaces SQL persistence": a store of facts and rules with a journal, driven by REXX. It has no SQL and no assembler.

- **Entry:** `DW-REXX-DISPATCH` evaluates `DW-RX-COMMAND`. The commands are OPEN, BEGIN, ASSERT, RETRACT, QUERY, UNIFY, BIND, UNBIND, CHOICE, BACKTRACK, RULE, EXECUTE, COMMIT, ROLLBACK and CLOSE. Anything else goes to `DW-FAIL-CLOSED` (status `FAILED`, return 999999).
- **Facts** have a predicate plus four arguments. `DW-ASSERT` rejects a fact with a blank predicate or first argument; otherwise it:
  1. assigns the next sequence number,
  2. builds the fact hash (`STRING` of predicate, arguments and sequence),
  3. indexes the predicate and first argument,
  4. journals an `ASSERT` entry and counts it in the transaction.
- **Query** counts index matches. A blank first argument matches any fact with that predicate.
- **Rules:** `DW-RULE-STORE` journals a rule head. `DW-RULE-EXECUTE` matches the head against the query predicate, pushes a choice point and unifies; if unification fails it backtracks. The status becomes `RULE-SATISFIED` on success.
- **Binding stack:** `DW-BIND`, `DW-UNBIND`, `DW-CHOICE` and `DW-BACKTRACK` move a depth counter capped at 999,999 and report `STACK-OVERFLOW` past it.
- **Journal:** every open, close, commit, rollback, assert, retract and rule step writes a typed journal entry with a new sequence number. `DW-SNAPSHOT` and `DW-RESTORE` mark and check a snapshot point.
- **Result to REXX:** `DW-EXPORT-RESULT` returns `STATUS|SEQUENCE|RESULT|MATCHES`.
- **Single-slot index:** the index holds one entry (`DW-INDEX` is not a table), so a query or retract compares against the most recently asserted fact.

### COBILT-DATAWORM-TREASURY — Dataworm plus ACH and treasury facts
**File:** `sources/C/Users/jessi/Desktop/bobs control repo/COBILT_DATAWORM_TREASURY.cob` (474 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/COBILT_DATAWORM_TREASURY.cob`

The same engine as COBILT-DATAWORM, with a proper main line (`DW-ENTRY`: initialize, dispatch, return, `GOBACK`) and two fact families on top:

| Command | Fact | Arguments |
|---|---|---|
| ACH-ASSERT / ACH-QUERY | `ach_entry` | entry id, batch id, trace, amount (packed S9(15)V99) |
| FUNDS-ASSERT / FUNDS-QUERY | `treasury_funds` | account, ledger balance, available, pending |

- `SNAPSHOT` stores the current sequence in `DW-EPOCH`, and `RESTORE` rolls the sequence back to it.
- A second dispatcher, `DW-REXX`, accepts the REXX spellings ASSERT-ACH, QUERY-ACH, ASSERT-FUNDS, QUERY-FUNDS, COMMIT and ROLLBACK.
- Every call ends with `DW-RETURN`, which formats `STATUS|SEQUENCE|RESULT|MATCHES` into `DW-RX-RESULT`.
- `DW-RX-COMMAND` sits in WORKING-STORAGE and there is no LINKAGE SECTION, so the command has to be set by code compiled into the same program. An outside caller has no way to pass one in.

### COBILT-ACH-TREASURY — ACH origination library
**File:** `sources/C/Users/jessi/Desktop/bobs control repo/COBILT-ACH-TREASURY.cbl` (484 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/COBILT-ACH-TREASURY.cbl`

The production library layer: REXX → COBOL → logic → DB2 → an external ACH adapter. Banking credentials stay outside the library.

**Interface:** `LK-REQUEST` (operation, batch id, entry id, 1024-byte payload) and `LK-RESPONSE` (status, packed code, message, object id = batch id).

**Operations**

| Operation | What happens |
|---|---|
| CREATE-BATCH | Opens a batch, zeroes the count and totals, and audits it. |
| ADD-ENTRY | Loads and validates the entry and checks idempotency; if new, adds the amount to the batch total and the control total, marks it `QUEUED`, and audits it. |
| VALIDATE-ENTRY | Requires entry id, routing id, account id and SEC code, an amount above zero, and a direction of `DEBIT` or `CREDIT`. |
| VALIDATE-BATCH | Requires at least one entry, batch total = control total, and the `batch_valid` predicate to hold. |
| CHECK-FUNDS | `funds_available`: available balance ≥ requested amount. |
| ROUTE-PAYMENT | `payment_route`: routing id present. |
| GENERATE-ACH | After batch validation, emits the five NACHA record types in order: 1 file header, 5 batch header, 6 entry detail, 8 batch control, 9 file control, each 94 characters. |
| SUBMIT | Batch valid and funds available → `EXTERNAL-ACH-ADAPTER` marks the batch `SUBMITTED` and audits it. |
| SETTLE | Allowed only from `SUBMITTED` → `SETTLED`. |
| RECONCILE | Compares expected and actual totals and counts → `RECONCILED`, otherwise `EXCEPTION`. |
| QUERY | Named predicates: `batch_valid`, `payment_valid`, `funds_available`, `authorized`. |
| UNIFY / BACKTRACK | Same term unification and choice-point pop as the other COBILT libraries. |
| COMMIT / ROLLBACK | `EXEC SQL COMMIT` / `EXEC SQL ROLLBACK` against DB2. |
| anything else | `FAIL-CLOSED`: status `FAILED`, code `UNKNOWN_OPERATION`, return 999999. |

**Integration points left as hooks:**
- `IDEMPOTENCY-LOOKUP` and `TREASURY-FUNDS-READ` are `CONTINUE`; the real lookups plug in there.
- `LOAD-ENTRY` copies only the entry id and payload, so routing, account and amount must be filled from the payload by the caller or a future parse step.
- RECONCILE currently copies the batch's own totals into both the expected and actual sides.

---

## 3. ACH returns and ledger posting

### ACHRTRN — ACH return processor (file-based)
**File:** `sources/C/Users/jessi/GolandProjects/devflow-finance-twin/finance/cobol/ACHRTRN.cbl` (392 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/ACHRTRN.cbl`

A called program that turns an ACH item into a return.

- **Files:** `ACHITEM` (indexed, key = company 3 + batch 10 + entry 15) holds the item, including its original trace, DFI, account, packed amount, debit/credit flag, state, dates, SEC code, addenda, and return fields. `ACHRETLOG` (indexed, key adds a log sequence) is the event log.
- **Request:** company, batch id, entry id, user id, channel, reason code. **Response:** success `Y`/`N`, error code, message, ledger sequence.
- **Flow:**
  1. Open both files; an open failure abends with `FILEOPEN`.
  2. Read the item; a missing item returns `NOTFOUND`.
  3. Check eligibility: the state must be `SETTLED` or `POSTED` (else `BADSTATE`), and the reason must be one of R01, R03, R04, R07, R08, R10, R29 (else `BADREASN`).
  4. Move the item to `RETURNED` and stamp reason, user, channel and timestamp. The return ledger sequence is left at 0 for the downstream posting engine.
  5. `REWRITE` the item (`DBERR` on failure).
  6. Log `RET_REQ` on success or `RET_FAIL` with the error text.
- **Reason codes:** R01 insufficient funds, R03 no account, R04 invalid account number, R07 authorization revoked, R08 payment stopped, R10 customer advises not authorized, R29 corporate customer advises not authorized.
- **Behavior details:**
  - The timestamp is the fixed value `20260908T192700.000Z+0000`; the comment marks it for a system service call.
  - An item already `RETURNED` is rejected by the state check as `BADSTATE`, so the later `ALREADYRT` branch is not reached.

### ACHMATRIX — ACH return as a transition matrix
**File:** `sources/C/Users/jessi/sovereign-engine-v2/constraint-harness/ACHRTRN.cbl` (549 lines)

The same decision expressed as matrix lookups ("no functor, pure array operations"):

- **States:** 1 NEW, 2 POSTED, 3 SETTLED, 4 RETURNED, 5 RETURN_POSTED, 6 CLOSED. The comment numbers them from 0, but the code indexes from 1.
- **6×6 transition matrix:**
  - Allowed moves: POSTED→RETURNED and SETTLED→RETURNED.
  - Self-loops on every state, for idempotency.
- **Reason vector:** 20 slots, of which slots 1–7 are the same R-codes as ACHRTRN.
- **Input:** state index, reason index, company, batch, entry. The target state is always 4 (RETURNED).
- **Output:** new state index, valid flag 1/0, and error code:
  - `OK`
  - `BADSTATE` (index out of 1–6)
  - `NOTALLOW` (matrix cell is 0)
  - `BADREASN` (reason slot invalid or out of range)

This file also carries a second WORKING-STORAGE, LINKAGE and PROCEDURE DIVISION after `STORE-OUTPUT-VECTOR`: the file-based ACHRTRN logic, without its FILE-CONTROL and FD entries. Treat it as ACHMATRIX with the ACHRTRN procedure appended.

### LEDGWYCB — reversal-posting callback
**File:** `sources/C/Users/jessi/GolandProjects/devflow-finance-twin/finance/cobol/LEDGWYCB.cbl` (51 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/LEDGWYCB.cbl`

A fixed-block call interface: 128-byte request in, 128-byte response out.

- **Request:** company, ledger date, ledger sequence, user, reason, channel, rail code.
- **Response:** success flag, error code, message, new ledger sequence.
- A blank company returns `BADREQ` / "Missing company".
- Otherwise it returns success, "Reversal posted", and ledger sequence `000001234`. That is where the internal COBOL/DB2 posting routine is called, per the comment.

### LEDGER_POST — ledger posting
**File:** `sources/C/Users/jessi/GolandProjects/devflow-finance-twin/finance/cobol/LEDGER_POST.cbl` (42 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/LEDGER_POST.cbl`

Posts one amount to an indexed `LEDGER` file:

- **Record:** company, date, sequence, packed S9(15)V99 amount, debit/credit flag.
- **Posting:** if the key is new, it writes a fresh record from `PARAM-*` values; if the key exists, it adds the amount to the stored total and writes it back.
- The `PARAM-*` fields and `LG-KEY` come from the wrapper that includes this code ("parameters mapped via LINKAGE or wrapper"); they are not declared in this file.

---

## 4. WORM ledger suite (worm-pow-rsa)

**Folder:** `sources/C/Users/jessi/RiderProjects/astra/worm-pow-rsa/cobol/`
(same five files also under `sources/C/Users/jessi/Desktop/sovereign-engine-v2/worm-pow-rsa/cobol/`)

A write-once, read-many (WORM) ledger. **Ada/SPARK upstream does all the cryptography** (payload hash, proof-of-work, RSA/signatures). The COBOL side only checks structure and never rewrites a ledger record.

### worm-record.cpy — shared record layout
One fixed-length ledger record of 2,459 bytes:

| Field | PIC | Meaning |
|---|---|---|
| WR-SCHEMA-ID | X(8) | schema version |
| WR-SEQUENCE | 9(18) | position in the chain, starting at 1 |
| WR-RECORD-ID | X(64) | unique record id |
| WR-TIMESTAMP | X(26) | ISO timestamp |
| WR-EVENT-TYPE | X(20) | event type |
| WR-PAYLOAD | X(2000) | payload |
| WR-PAYLOAD-HASH | X(64) | hex hash of the payload |
| WR-PREV-HASH | X(64) | previous record's WR-RECORD-HASH |
| WR-POW-ALGORITHM / NONCE / DIFFICULTY / HASH | X(16) / 9(18) / 9(9) / X(64) | proof-of-work |
| WR-RECORD-HASH | X(64) | this record's hash |
| WR-WORM-MODE | X(12) | mode label |
| WR-WORM-APPEND-ONLY / IMMUTABLE | X / X | must both be `Y` |

### APPEND-RECORD — single-record append
- Reads `INCOMREC` and opens the ledger `WORMLDGR` in **EXTEND mode only**, so it cannot rewrite or delete.
- Appends each incoming record whose append-only and immutable flags are both `Y`, and rejects the rest.
- Prints read/appended/skipped counts.

### BATCH-INGEST — bulk append with chain checks
1. Reads the whole existing ledger to find the last sequence and record hash. A missing ledger counts as empty.
2. For each record in `INGEST`:
   - the sequence must be the prior sequence + 1 (else `SEQGAP`);
   - unless the ledger is empty, `WR-PREV-HASH` must equal the prior record hash (else `HASHMIS`).
3. Good records are appended in EXTEND mode and become the new "prior". Bad records go to `EXCEPT`, with a reason code and the original record.
4. A bad record never stops the batch. It ends with read/appended/rejected counts.

### RECONCILE-LEDGER — full-chain audit (fail-closed)
- Reads the ledger in order and checks each record for:
  - a sequence gap (the chain must start at 1),
  - a broken hash link to the previous record,
  - a duplicate sequence number,
  - a duplicate record id.
- Duplicates are found by scanning a table of up to 50,000 seen entries. Going past that limit counts as an anomaly.
- Each failure writes a detail line to `RECONRPT` and flips the run to FAIL, which is never reset.
- The report ends with counts per check and `OVERALL RESULT: PASS` or `FAIL`.

### ARCHIVE-SEGMENT — archive with manifest
Makes two read-only passes over the ledger:
1. The first pass computes the record count, the first and last sequence, and the first and last record hash.
2. The second pass opens `ARCHIVE` once for output and writes a `MANIFEST01` header line followed by every ledger record, in order.

The chain id defaults to `DEFAULT-CHAIN`. The ledger is never modified.

---

## 5. Bridges to Rust and PL/I

### SNAPKITTY-TRANSACTION-BRIDGE — transaction gate for Rust
**File:** `sources/C/Users/jessi/Desktop/THE_BOOK/bridges/cobol/transaction.cbl` (111 lines)
also at `sources/C/Users/jessi/Desktop/SKC/DEVFLOW-FINANCE/bridges/cobol/transaction.cbl`

A SUBLEQ(A, B, C) gate: A is the amount, B is a zero threshold, and C routes the transaction to the Rust WORM seal or rejects it.

1. **100-LOAD-ENV:** reads `TX_ID`, `TX_AMOUNT`, `TX_SOURCE`, `TX_DEST`, `TX_TIMESTAMP` and `TX_CATEGORY` from environment variables that Rust sets before invoking. The amount is converted with `NUMVAL`.
2. **200-VALIDATE:** a blank id → `INVALID`; amount ≤ 0 → `INVALID`; otherwise `APPROVED`.
3. **300-SUBLEQ-GATE:** an invalid transaction becomes `REJECTED`.
4. **400-OUTPUT-JSON:** prints one JSON object (id, amount, source, dest, category, timestamp, status, `"bridge":"cobol-1959"`), which Rust parses and routes to the WORM chain.

The copy at `sources/C/Users/jessi/Desktop/bobs control repo/forge-legacy-bridge/cobol/transaction.cbl` has lines 2–104 overwritten with Rust source (a `ContextHydrator` module), so only its tail is COBOL. Use the THE_BOOK copy.

### TRSY-WORM-BRIDGE — treasury records into WORM blocks
**File:** `sources/C/Users/jessi/Desktop/devflow-finance-twin-fix/cobol/worm_bridge.cob` (59 lines)
also at `sources/D/tmp/devflow-finance-twin/cobol/worm_bridge.cob`

DEED-089, the Sovereign Treasury Engine, AGPL-3.0-or-later. It bridges PL/I output into WORM storage.

- **Record layouts:**
  - Treasury record: transaction id, packed timestamp, sequence, source and destination accounts, packed amount, currency, compliance flag.
  - WORM block header: magic `WORM`, previous hash, current hash, record count.
  - 4 KB serialization buffer.
- **Flow:** reads `TREASURY.DAT` until end of file. For each record, `SERIALIZE-AND-SEAL` copies the transaction id into the payload, moves the buffer into the block header, writes it to `WORM.ESDS`, and counts it.
- The file names are held in data items rather than `SELECT`/`FD` entries, so the file binding comes from the build around it.

### SOV-RECORD-GATE — density-matrix record gate for the PL/I kernel
**File:** `sources/C/Users/jessi/SNAPKITTYWEST/sov-kernel-monster/sovereign-pli/sov_record_gate.cbl` (129 lines)
also at `sources/D/tmp/sov-kernel-monster-rtx-20260725/sovereign-pli/sov_record_gate.cbl`

It is called from `sov_kernel.pli` with a 512-byte record buffer, a length, and a return code, and is interlocked with `intercal_invert.i`.

- **Record:** generation, φ-energy (fixed-point), matrix dimension, trace sum, Blake3 hash (32), Ed25519 signature (64), WORM-sealed flag, and 8 eigenvalues (real and imaginary parts).
- **Steps:**
  1. **200-VALIDATE-DENSITY:** sums the real eigenvalues. A negative eigenvalue gives code 2, and a sum other than 1.0 gives code 1. Together these are the density-matrix conditions: trace = 1 and positive semidefinite.
  2. **300-APPLY-PHI-DECAY:** multiplies the energy by φ⁻¹ = 0.6180339887 and increments the generation. This mirrors the Thermal Monad bind in LiquidLean.
  3. **400-WORM-SEAL-CHECK:** code 3 if the record is not sealed.
  4. **500-ENQUEUE-STATE:** pushes the record into a 256-slot ring buffer and drops it when the buffer is full (non-blocking).
- **Return value:** the validation code, 0 when valid. When several checks fail, the last one to run sets the code.
- The flat PERFORM structure is deliberate: no recursion, no stack growth.

---

## 6. Agent and kernel programs

### VAULT-TREASURY — treasury agent
**File:** `sources/C/Users/jessi/Desktop/THE_BOOK/bridges/cobol/vault_treasury.cbl` (74 lines)
also at `sources/C/Users/jessi/Desktop/SKC/DEVFLOW-FINANCE/bridges/cobol/vault_treasury.cbl`

Its axiom is "capital never moves without a seal". It reads one query line from stdin and lowercases it.

1. **Reserve check:** the reserve (75) must be above the threshold (20), or the check fails and the status is `FROZEN`. Both values are constants in the program, so the check passes as written.
2. **Intent**, from the start of the query:
   - `freeze…` → `FROZEN`
   - `approve…` with the reserve check passing → `SEALED`
   - `release…` with the reserve check failing → `FROZEN`
3. **Output:** `key=value` lines covering agent, status, reserve check, approval required, veto power, a decision sentence, `certified=true` and the engine name.

### COBOL-LAW-KERNEL — legal claim router
**File:** `sources/D/tmp/cartographer-agent/kernels/cobol-law-kernel.cob` (46 lines)

It prompts for a case id and a claim type, maps the claim to a ledger code, notice code and route, and prints them:

| Claim type | Ledger | Notice | Route |
|---|---|---|---|
| BENEFICIARY | TRUST-BENEFICIARY | NOTICE-REPRESENTATION | FIDUCIARY-REVIEW |
| ACH | ACH-DISPUTE | NOTICE-REVOCATION | RDFI-ODFI-PATH |
| FCRA | CREDIT-DISPUTE | NOTICE-METRO2 | FURNISHER-CRA-PATH |
| other | GENERAL-LEGAL | NOTICE-GENERAL | COUNSEL-REVIEW |

### MAMARI-TABLET-DECODER — lunar calendar processor
**File:** `sources/C/Users/jessi/Desktop/the-49th-call/substrate/mamari.cbl` (109 lines)
also at `sources/D/tmp/sovereign-engine-v2/the-49th-call/substrate/mamari.cbl`

It treats the Mamari Tablet (Easter Island, about 800 CE) as a 30-row structured record and applies the OISC idea: SUBLEQ over a lunar counter.

- **Input:** reads `MAMARI.DAT` line by line. Each record has a position (2), glyph id (6), lunar phase (12), adjacent glyph (6) and confidence (9V99).
- **Thresholds:** it counts phases, and at position 15 (full moon) and 29 (dark moon) prints a phase transition.
- **The "C operand"** is the adjacent glyph, which decides the instruction: FISH → "SET THE NETS", BIRD → "OBSERVE THE SKY", anything else is printed as-is.
- **Summary:** total phases and the last instruction that fired.

---

## 7. Copybooks (record layouts only)

### budget-loc.cpy — budget variance and lines of credit
**File:** `sources/C/Users/jessi/IdeaProjects/DEVFLOW-FINANCE/polyglot/implementations/cobol/budget-loc.cpy`
also at `sources/C/Users/jessi/SNAPKITTYWEST/sovereign-suite/polyglot/implementations/cobol/budget-loc.cpy`

| Record | Holds |
|---|---|
| BUDGET-RECORD | id, name, entity, fiscal year, status (DRAFT / APPROVED / ACTIVE / CLOSED), total in cents |
| BUDGET-LINE-ITEM-RECORD | budget line per account and period (YYYY-MM), amount, notes |
| BUDGET-VARIANCE-RECORD | budget vs. actual per account and period, signed variance and %, over-budget flag, CRITICAL / WARNING / OK |
| VARIANCE-THRESHOLD-RECORD | critical %, warning %, tolerance in cents |
| VARIANCE-REPORT-RECORD | per-budget totals and over / under / on-track / critical counts |
| LOC-AUDIT-RECORD | line of credit: limit, available, used, rate in basis points, FIXED / VARIABLE / PRIME_PLUS, maturity, status (ACTIVE / FROZEN / DEFAULT / PAID_OFF / EXPIRED) |
| COVENANT-CHECK-RECORD | covenant type, threshold vs. actual, PASS / FAIL / WAIVED, trend |

### consolidation.cpy — multi-entity consolidation
**File:** `sources/C/Users/jessi/IdeaProjects/DEVFLOW-FINANCE/polyglot/implementations/cobol/consolidation.cpy`
also at `sources/C/Users/jessi/SNAPKITTYWEST/sovereign-suite/polyglot/implementations/cobol/consolidation.cpy`

| Record | Holds |
|---|---|
| ENTITY-RECORD | entity id, code, name, type (NONPROFIT / BCORP / TRUST), parent, currency, ACTIVE / INACTIVE |
| IC-TRANSACTION-RECORD | intercompany transaction: LOAN / ALLOCATION / SERVICE_FEE / DIVIDEND / TRANSFER, from/to entity, amount, PENDING / APPROVED / ELIMINATED / VOID |
| CONSOLIDATION-RULE-RECORD | ELIMINATION / INTERCOMPANY / CURRENCY_TRANSLATION / RECLASSIFICATION, from/to account, percentage |
| CONSOLIDATION-RESULT-RECORD | period, entity count, elimination count, consolidated balance, DRAFT / COMPLETED / FAILED |
| CURRENCY-TRANSLATION-RECORD | from/to currency, rate (9(3)V9(8)), effective date, source |

### SEBEVENT.cpy — Sovereign Event Bus envelope (RPG, not COBOL)
**File:** `sources/D/tmp/Sovereign-Event-Bus/seb/adapters/SEBEVENT.cpy`

This is an **RPG ILE copybook** (D-specs) for IBM i. It was collected because of its `.cpy` extension. It defines the envelope that the Sovereign Event Bus exchanges with IBM i:

- **Header:** WORM-chain offset, ISO-8601 timestamp, agent id (for example FISCAL_SETTLE or BIFROST), event type (SETTLEMENT, ROUTING, VERIFY), payload size, and a reserved area.
- **Footer (the seal):** previous Blake3 hash, this event's Blake3 hash (over header + payload + previous hash), and an Ed25519 signature of the event hash.
- **Payload:** JSON (intent, context, authority, continuation, evidence), stored separately as `SEB_PAYLOAD_<offset>_<agent>.blob`, up to 2 GB.
- **External programs:**
  - `SEB_APPEND` appends an event and returns its offset.
  - `SEB_VERIFY` checks chain integrity over an offset range.
  - `SEB_READ` reads an event by offset.
  - `SEB_COMMIT` commits an offset against a Bifrost hash, which gives idempotency: a repeated settlement returns the existing offset.
- **Error codes 0–8:** success, invalid offset, invalid size, hash mismatch, invalid signature, file read, file write, chain broken, payload corrupt.

---

## 8. Cobalt: REQPARSE and the bank suite

### REQPARSE — HTTP request-line parser (nugget N07)
**File:** `cobalt/REQPARSE.cbl` (159 lines). It is the first vertical slice of the recursive COBOL nugget compiler, built from the "Build your own Web Server" family of the build-your-own-x corpus.

- **Input:** one line from `ACCEPT`, for example `GET /index.html HTTP/1.1`. The length is the position of the last non-space character.
- **Nuggets:**

| Nugget | Paragraph | Rule |
|---|---|---|
| N01 | VALIDATE-BUFFER | Length must be 1–512. |
| N02–N05 | FIND-TOKENS | Split on spaces into method, target and version. More than two spaces, or fewer than two, fails. |
| N03 | VALIDATE-METHOD | GET, POST, HEAD, PUT or DELETE. |
| N04 | VALIDATE-TARGET | Non-blank and starts with `/`. |
| N06 | VALIDATE-VERSION | Exactly `HTTP/1.0` or `HTTP/1.1`. |
| N07 | PARSE-REQUEST-LINE | All checks pass → `APPROVED`, otherwise `DECLINED`. |

- **Output:** four lines: `STATUS=`, `METHOD=`, `TARGET=`, `VERSION=`.

### Bank suite
**Folder:** `cobalt/bank-suite/`

| File | What it does |
|---|---|
| BANK2024.cob | Reads `accounts.dat` sequentially (number, name, balance, status A/F/C). It skips records with a zero account number, an overdraft beyond −10,000, or an unknown status. It prints each account and a summary: count, active count, total, maximum and mean balance. The return code is 0, or 4 when there were warnings. |
| BANKIDX.cob | CRUD over an indexed `bankidx.dat`, with account number as the primary key and name as an alternate key (duplicates allowed). The operation comes from the command line: CREATE seeds four accounts, and READ, UPDATE (+100.00), DELETE, LIST and SEEKNAME take an argument. |
| BANKJSON.cob | `JSON GENERATE` of an account group, then `JSON PARSE` back into a second group, then a field-by-field round-trip check. |
| BANKOO.cob | OO COBOL: `BankAccount` (constructor, deposit, withdraw with a funds check, getBalance, finalize) and `SavingsAccount`, which inherits it and overrides withdraw to block breaching a minimum balance. |
| BANKTEST.cob + COBUNIT.cpy | Pure-COBOL unit tests, using the COBUNIT counters, for withdrawal rules (sufficient, exact, insufficient, frozen), status 88-levels, and numeric round-trips. Return code 0 means everything passed. |
| run-tests.sh, Makefile | Build all programs with `cobc -x -std=ibm -free` and run the report, CRUD, JSON, OO and unit suites with pass/fail counts (`make test`). |

### hello.cob — transformer test fixture
**File:** `cobol-transformer/tests/fixtures/hello.cob` (13 lines). This is the input for the transformer's integration test: identification, working-storage with a PICTURE X(20) VALUE item, and DISPLAY / STOP RUN.

---

## 9. Notes for the transformer

These constructs appear in this corpus, so the transformer's lexer and parser need to handle them:

- **Free-format comment markers:** `*>` both in column 7 and inline, alongside fixed-format `*` in column 7. `budget-loc.cpy` and `consolidation.cpy` put `*>` in column 8.
- **`EXEC SQL … END-EXEC`** blocks (COBILT-ACH-TREASURY).
- **`SPECIAL-NAMES. DECIMAL-POINT IS COMMA`** (SOV-RECORD-GATE), which changes how numeric literals such as `0.6180339887` must be read.
- **Unquoted words as `EVALUATE` subjects and `VALUE` clauses** in COBILT-VAULT (`WHEN VAULT-OPEN`, `VALUE IS READY`). These parse as identifiers, not literals.
- **Several divisions in one file** (ACHMATRIX with the ACHRTRN procedure appended) and a second WORKING-STORAGE SECTION after the procedure division (BANKIDX).
- **`COPY WORM-RECORD`** inside an `01` level and inside a nested `05` group (BATCH-INGEST's exception record), so COPY must splice at any level.
- **OO syntax:** `CLASS-ID`, `METHOD-ID`, `INVOKE`, `INHERITING FROM`, `OBJECT REFERENCE` (BANKOO).
- **`JSON GENERATE` / `JSON PARSE`** with `ON EXCEPTION` (BANKJSON).
- **`ACCEPT … FROM ENVIRONMENT`** and **`FROM STDIN`** (transaction bridge, VAULT-TREASURY).
- **`NEXT SENTENCE`** inside IF/ELSE, and `OCCURS … INDEXED BY` tables.
- **A non-COBOL file** with a COBOL extension (SEBEVENT.cpy is RPG), plus Rust text inside a `.cbl` (the forge-legacy-bridge copy of transaction.cbl). The collector picks files by extension, so the transformer should detect these and skip them.
