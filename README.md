# COBOL Transformer Repo

One place for the COBOL transformer and every COBOL source collected from the
local drives.

## Layout

| Path | Contents |
|---|---|
| `cobol-transformer/` | Bob's Rust COBOL transformer: lexer, parser, AST, codegen, CLI, and analysis modules (symbols, types, CFG, dataflow, transforms). Build with `cargo build`; see its README. |
| `cobalt/` | Cobalt, the hand-rolled Haskell compiler, plus `REQPARSE.cbl` (HTTP request-line nugget N07) and `bank-suite/`. |
| `sources/` | COBOL files collected from C:\ and D:\, stored under their original absolute paths (`sources/C/...`, `sources/D/...`). |
| `transformer/` | The first collection's copy of the transformer at its mirrored path. Superseded by `cobol-transformer/`. |
| `MANIFEST.csv` | Every collected file: source path, repo path, size, SHA-256, last write time, and whether it was copied or was a duplicate of an earlier copy. |
| `docs/COBOL-PROGRAMS.md` | What every COBOL program does. |

## COBOL programs at a glance

Full detail is in [docs/COBOL-PROGRAMS.md](docs/COBOL-PROGRAMS.md).

| Program | System | Purpose |
|---|---|---|
| COBILT-VAULT | COBILT (IBM i) | Key/value vault with authority checks, hash chaining, and a logic engine (facts, rules, unify, backtrack); REXX/RPGLE/COBOL bridge. |
| COBILT-DATAWORM | COBILT | Datalog fact/rule store with a journal, replacing SQL persistence. |
| COBILT-DATAWORM-TREASURY | COBILT | Dataworm plus `ach_entry` and `treasury_funds` facts, snapshot/restore. |
| COBILT-ACH-TREASURY | COBILT | ACH origination: batches, entry validation, funds check, NACHA 1/5/6/8/9 records, submit, settle, reconcile, DB2 commit. |
| ACHRTRN | ACH | Returns an ACH item: SETTLED/POSTED → RETURNED with NACHA reason codes; indexed item file and event log. |
| ACHMATRIX | ACH | The same return decision as a 6×6 transition-matrix lookup. |
| LEDGWYCB | ACH | 128-byte request/response callback for posting reversals. |
| LEDGER_POST | Ledger | Posts or accumulates an amount on an indexed ledger. |
| APPEND-RECORD, BATCH-INGEST, RECONCILE-LEDGER, ARCHIVE-SEGMENT (+ worm-record.cpy) | WORM ledger | Append-only ledger. Crypto is done upstream in Ada/SPARK; COBOL enforces sequence and hash-link continuity, reconciles fail-closed, and archives with a manifest. |
| SNAPKITTY-TRANSACTION-BRIDGE | Rust bridge | Env-var transaction → validate → SUBLEQ gate → JSON for the Rust WORM seal. |
| TRSY-WORM-BRIDGE | PL/I → WORM | Serializes treasury records into WORM blocks. |
| SOV-RECORD-GATE | PL/I kernel | Validates density-matrix records (trace = 1, no negative eigenvalues, sealed), applies φ⁻¹ decay, and queues them. |
| VAULT-TREASURY | Agent | stdin query → reserve check → SEALED/FROZEN decision as `key=value` lines. |
| COBOL-LAW-KERNEL | Agent | Routes a claim (BENEFICIARY, ACH, FCRA) to ledger, notice, and review path. |
| MAMARI-TABLET-DECODER | the-49th-call | Lunar glyph records processed as SUBLEQ thresholds (full/dark moon). |
| budget-loc.cpy, consolidation.cpy | DEVFLOW-FINANCE | Record layouts for budget variance, lines of credit, covenants, and multi-entity consolidation. |
| SEBEVENT.cpy | Event bus | RPG copybook for the Sovereign Event Bus envelope (Blake3 + Ed25519 seal). |
| REQPARSE | Cobalt | HTTP/1.1 request-line parser built from nuggets N01–N07. |
| BANK2024, BANKIDX, BANKJSON, BANKOO, BANKTEST | Cobalt bank suite | Sequential report, indexed CRUD, JSON round-trip, OO classes, and pure-COBOL unit tests. |
