# COBOL Transformer Repo

One place for the COBOL transformer and every COBOL source collected from the
local drives.

## Layout

| Path | Contents |
|---|---|
| `cobol-transformer/` | Bob's Rust COBOL transformer: lexer, parser, AST, codegen, CLI, and analysis modules (symbols, types, CFG, dataflow, transforms). Build with `cargo build`; see its README. |
| `cobalt/` | Cobalt, the hand-rolled Haskell compiler, plus `REQPARSE.cbl` (HTTP request-line nugget N07) and `bank-suite/`. |
| `sources/` | COBOL files collected from C:\ and D:\, stored under their original absolute paths (`sources/C/...`, `sources/D/...`). |
| `web/` | Browser playground: the transformer compiled to WebAssembly (`web/wasm/`, wasm-bindgen) with a Tailwind UI (`web/src/`). Built site in `web/dist/`. See [Web playground](#web-playground) and [docs/HACKATHON.md](docs/HACKATHON.md). |
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

## Web playground

`web/` runs the real Rust transformer in the browser. The `cobol-transformer`
library is compiled to WebAssembly through a small wasm-bindgen wrapper
(`web/wasm/`), and a plain HTML + JS page styled with Tailwind CSS calls it.
No COBOL logic is written in JavaScript. Details, architecture and limitations:
[docs/HACKATHON.md](docs/HACKATHON.md).

What you can do on the page:

- pick a sample (real files from this repo: `cobol-transformer/tests/fixtures/hello.cob`,
  `cobalt/REQPARSE.cbl`, programs under `sources/`, and `gpu-cobol/examples/vector-add.cbl`)
  or paste your own source;
- run any operation the CLI has: parse, dump AST, dump symbols, dump CFG, token
  stream, generate COBOL with the `normalize` / `modernize` passes, round-trip
  validation, plus format detection and fixed/free normalisation;
- view output as a collapsible tree, token table, raw text, or the JSON the wasm returns;
- see lexer and parser errors with line and column, and jump to them;
- switch dark/light; the layout works at phone width.

### Build

Prerequisites: Rust (stable), `rustup target add wasm32-unknown-unknown`,
[wasm-pack](https://github.com/rustwasm/wasm-pack) on `PATH`, Node.js 20+.

```bash
cd web
npm ci                 # installs tailwindcss + @tailwindcss/cli only
npm run build          # = build:wasm + build:site + build:css
npm run verify         # loads dist/pkg in Node, calls every export, serves dist under a sub-path
```

The individual steps are:

```bash
wasm-pack build wasm --target web --release --out-dir pkg   # npm run build:wasm
node scripts/build-site.mjs                                  # npm run build:site  -> web/dist
tailwindcss -i src/styles.css -o dist/app.css --minify       # npm run build:css
```

### Run locally

`web/dist` is a static site with only relative URLs, so it works from any
static host and any sub-path. Serve it over HTTP (browsers will not load
`.wasm` modules from `file://`):

```bash
npx serve web/dist                         # or
python -m http.server 8080 -d web/dist     # or
cd web && npm run serve                    # built-in server, http://127.0.0.1:8080/
```

Deep links: `index.html?sample=ledger-post&op=generate`.

### GitHub Pages

`.github/workflows/pages.yml` builds the wasm and CSS, runs `npm run verify`,
and deploys `web/dist` to GitHub Pages on pushes to `main` that touch
`cobol-transformer/`, `web/` or the workflow. **This repository is private and
the account is on GitHub Free, where Pages does not publish private
repositories.** The build and verify jobs still run, but the deploy step will not
publish until the repository is made public or the account is upgraded. Until
then, use the committed `web/dist` with any static host or locally.
