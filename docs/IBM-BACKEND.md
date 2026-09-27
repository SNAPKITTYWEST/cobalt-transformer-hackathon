# IBM watsonx.ai backend

Two independent, overlapping implementations of the same idea — ground
watsonx.ai's Granite models on facts pulled from the real COBOL parser, rather
than raw text — built by different people at different times. Neither
depends on the other; the field guide's five-service stack
(`docs/IBM-HACKATHON-FIELD-GUIDE.md`) applies to both.

| | Rust module | Python service |
|---|---|---|
| Location | `cobol-transformer/src/ibm/` | `service/cobalt_service/` |
| Runs as | part of the `cobol-transform` crate/binary | a standalone FastAPI process |
| Grounding | passes raw COBOL source to Granite | runs `facts/` (a small Rust binary linking the parser) to extract paragraphs, data items, symbols, and CFG edges first, then grounds the prompt on those facts |
| Status | builds clean, 11/11 unit tests pass (see below) | written, endpoints defined, not yet started/tested end-to-end |

Both need live IBM Cloud credentials to actually call watsonx.ai; neither has
any on this machine (see **Credentials**, below). Everything gated on
credentials is clearly marked `ignored` (Rust) or fails fast with a
configuration error (Python) — nothing fakes a response.

## Rust: `cobol-transformer/src/ibm/`

Origin: written directly against `cobol-transformer` (the "TRITON" engine),
copied in file-by-file from the working IBM-integration branch and rebuilt
against this repo's copy of the crate (`cargo build --release`: clean;
`cargo test --test ibm_integration_test`: 11 passed, 0 failed, 5 ignored).

| File | Lines | Contents |
|---|---|---|
| `auth.rs` | 130 | IAM token exchange (`POST https://iam.cloud.ibm.com/identity/token`) and caching |
| `watsonx.rs` | 349 | `WatsonxClient`: chat/explain, code translation, test generation, embeddings, against `/ml/v1/text/{chat,embeddings}` |
| `client.rs` | 349 | `ObjectStorageClient` (S3-compatible HMAC), `CodeEngineClient` (deploy helpers) |
| `mod.rs` | 123 | `TritonIbmIntegration`: wires the three together into `process_cobol_program()` — upload source → explain → (store result → deploy) |
| `examples/ibm_integration_demo.rs` | 219 | four runnable demo scenarios, configured from environment variables |
| `tests/ibm_integration_test.rs` | 282 | 11 unit tests (construction/config, no network) + 5 integration tests marked `#[ignore]` pending credentials |

Run the demo (needs credentials — see below):
```powershell
cd cobol-transformer
$env:IBM_API_KEY="..."; $env:IBM_PROJECT_ID="..."; $env:IBM_COS_BUCKET="..."
cargo run --example ibm_integration_demo
```
Run just the tests that don't need credentials:
```powershell
cargo test --test ibm_integration_test
```

## Python: `service/cobalt_service/`

Origin: written for this repo, calling the real parser through a small
companion Rust binary (`facts/`, see below) instead of embedding libcobol
logic in Python.

| Module | Responsibility |
|---|---|
| `config.py` | settings from environment variables only (`WATSONX_APIKEY`, `WATSONX_PROJECT_ID`, `WATSONX_URL`, `COS_*`) |
| `iam.py` | IAM token cache/refresh (same flow as the Rust `auth.rs`) |
| `ratelimit.py` | enforces the Lite plan's 2 requests/second cap |
| `usage.py` | per-request and per-process token accounting |
| `watsonx.py` | REST calls to chat, embeddings, rerank, and the model catalog, with 401/429 retry |
| `storage.py` | Object Storage over HMAC, or a local directory clearly labeled `local-filesystem` when COS isn't configured |
| `parser_bridge.py` | shells out to `facts/`, the Rust binary below |
| `chunking.py` | paragraph-level chunks from the parser; where the parser stops early, falls back to a regex splitter — every chunk records which method produced it |
| `prompts.py` | few-shot prompts for Java/Python translation, grounded in the facts from `parser_bridge.py` |
| `sandbox.py` | runs generated code in a scrubbed-environment subprocess with a timeout and POSIX rlimits (Python, Java, and `cobc` targets) |
| `vectors.py` | cosine similarity / nearest-neighbor over embedding vectors |
| `pipeline.py` | ties it together: ingest → index → search/rerank → duplicate detection → explain → transform → validate |
| `app.py` | the FastAPI app |

### Endpoints (`app.py`)

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/health` | liveness |
| GET | `/api/usage` | token usage so far |
| GET | `/api/models` | watsonx.ai model catalog passthrough |
| POST | `/api/parse` | run the real parser (via `facts/`) on posted source |
| POST | `/api/ingest`, `/api/ingest/upload` | add a program to an estate (COS or local) |
| GET | `/api/estates/{id}` | estate metadata |
| POST | `/api/estates/{id}/index` | embed and index the estate's paragraphs |
| POST | `/api/estates/{id}/search` | semantic search + rerank over the estate |
| GET | `/api/estates/{id}/duplicates` | duplicate-logic detection via embeddings |
| POST | `/api/explain` | Granite explanation of a paragraph, grounded on parser facts |
| POST | `/api/transform` | Granite translation to Java/Python |
| POST | `/api/validate` | generate + sandbox-run tests for a transformed unit |

### `facts/` — the grounding bridge

A small standalone Rust binary (`facts/src/main.rs`, links `cobol-transformer`
as a library) that runs the real lexer/parser/CFG builder over a COBOL file
and prints JSON: paragraph names with line ranges, data items (PIC, USAGE,
VALUE, parent group), symbol-table entries, and PERFORM/GO TO control-flow
edges. `chunking.py` calls this instead of re-parsing COBOL in Python, so the
"grounding" is the same parser that powers the web playground, not a second,
divergent implementation.

Built and run against the collected samples (`cargo build --release`, 29.5s):

| Sample | Result |
|---|---|
| `hello.cob` | fully parsed — `MAIN-PARA` at lines 8–11, statements `[Display, Stop]`, refs `[GREETING]` |
| `ledger-post.cbl` | parses to line 5 (`INPUT-OUTPUT`), then stops |
| `law-kernel.cob` | parses to line 9, then stops |
| `reqparse.cbl` | lexer error at 6:39 (em dash in a comment) |
| `vault-treasury.cbl` | lexer error at 33:27 (`<`) |
| `mamari.cbl` | lexer error at 4:9 (`=`) |

These are the same gaps recorded in `docs/HANDOFF.md` §3.2 for the web
playground, because both call the same crate.

### Status

Written and gitignored-appropriately (`service/.venv/`, `service/.data/`,
`.env` are all excluded — see the root `.gitignore`), **not yet**: unit
tested, started locally, exercised end to end, containerized, or committed.
`facts/` builds and its output above is real; `service/` itself has not been
run.

## Credentials

Neither implementation has live IBM Cloud credentials on this machine. Both
read them from environment variables only — never hardcode a key or project
ID in either the Rust or Python code:

| Variable | Used by |
|---|---|
| `IBM_API_KEY` / `WATSONX_APIKEY` | IAM token exchange (Rust / Python name differs; unify when wiring a shared `.env`) |
| `IBM_PROJECT_ID` / `WATSONX_PROJECT_ID` | watsonx.ai Studio project |
| `WATSONX_URL` | region endpoint, default `https://us-south.ml.cloud.ibm.com` |
| `IBM_COS_BUCKET`, `COS_ENDPOINT`, `COS_HMAC_ACCESS_KEY_ID`, `COS_HMAC_SECRET_ACCESS_KEY` | Object Storage |

See `docs/IBM-HACKATHON-FIELD-GUIDE.md` → "Get access in five moves" for how
to obtain these on the free Lite plan.

## Do not repeat unmeasured numbers

An earlier draft of the hackathon deployment plan quoted fixed speedup figures
(6.2x, "4–8x") for a vectorization analyzer. Those numbers are hardcoded
constants in that analyzer's estimator, not measurements — the analyzer
returns a fixed `8 × 0.8` or `iterations × 0.8` regardless of input, and its
loop-carried-dependency check always reports "safe". If a vectorization
analyzer is presented in the demo, label its output as an *estimate from the
analyzer's model*, not a measured result, unless it has actually been
benchmarked against real scalar/vector execution.
