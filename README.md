# Cobalt COBOL Transformer

**Start here:** [SUBMISSION.md](SUBMISSION.md) — what this project is, what
works, what's honestly incomplete, and the live demo link. This README is the
repository layout reference; `SUBMISSION.md` is the narrative entry point.

## Layout

| Path | Contents |
|---|---|
| `cobol-transformer/` | Bob's Rust COBOL transformer: lexer, parser, AST, codegen, CLI, analysis modules (symbols, types, CFG, dataflow, transforms), and `src/ibm/` (watsonx.ai/Object Storage/Code Engine client). Build with `cargo build`. See [docs/IBM-BACKEND.md](docs/IBM-BACKEND.md). |
| `gpu-cobol/` | A second, independent compiler: a COBOL-syntax dialect for GPU kernels, hand-written in C, emitting PTX. See [docs/GPU-COBOL.md](docs/GPU-COBOL.md). |
| `cobalt/` | Cobalt, the hand-rolled Haskell compiler, plus `REQPARSE.cbl` (HTTP request-line nugget N07), `bank-suite/`, and an `attention/` kernel implementation. |
| `sources/` | 37 real COBOL programs collected from other projects, stored under their original absolute paths (`sources/C/...`, `sources/D/...`), used as a test corpus. Cataloged in [docs/COBOL-PROGRAMS.md](docs/COBOL-PROGRAMS.md). |
| `web/` | Browser playground: `cobol-transformer` compiled to WebAssembly (`web/wasm/`, wasm-bindgen) with a Tailwind UI (`web/src/`). Built site in `web/dist/`, live at the [GitHub Pages demo](#web-playground). See [docs/HACKATHON.md](docs/HACKATHON.md). |
| `facts/`, `service/` | The Python/FastAPI half of the IBM backend: a Rust "facts" bridge plus a service that grounds watsonx.ai prompts on real parser output. Not yet run end to end. See [docs/IBM-BACKEND.md](docs/IBM-BACKEND.md). |
| `transformer/` | An earlier, superseded copy of the transformer at its original mirrored path, kept for the collection's provenance. Not part of the working submission — use `cobol-transformer/`. |
| `MANIFEST.csv` | Every collected file: source path, repo path, size, SHA-256, last write time, and whether it was copied or was a duplicate of an earlier copy. |
| `docs/` | All project documentation — see the map in [SUBMISSION.md](SUBMISSION.md#documentation-map). |

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
`cobol-transformer/`, `web/` or the workflow, and on manual runs
(Actions → "Web playground (GitHub Pages)" → Run workflow). The live demo is at
**https://snapkittywest.github.io/cobalt-transformer-hackathon/**.
