# Hackathon edition: COBOL Transformer in the browser

## The project

This repository collects COBOL sources from local drives (see
[COBOL-PROGRAMS.md](COBOL-PROGRAMS.md)) together with `cobol-transformer/`, a
Rust COBOL source-to-source transformer: preprocessor, lexer, recursive-descent
parser, typed AST, symbol table, control-flow-graph builder, transform engine
and code generator, driven by the `cobol-transform` CLI.

The hackathon edition adds a web playground in `web/` so the transformer can be
tried without installing Rust: open a page, pick or paste COBOL, run an
operation, and read the result.

## Architecture

```
cobol-transformer/            Rust library + CLI (unchanged pipeline)
        |  path dependency
        v
web/wasm/  (cobol-transformer-wasm)
        wasm-bindgen exports -> JSON strings
        |  wasm-pack build --target web
        v
web/wasm/pkg/                 cobol_transformer_wasm.js + _bg.wasm
        |  scripts/build-site.mjs copies pkg, page, samples
        v
web/dist/                     index.html, app.js, app.css (Tailwind CLI),
                              pkg/, samples/ (real repo files + samples.json)
```

1. **Rust transformer.** `web/wasm/src/lib.rs` calls the library exactly as
   `cobol-transformer/src/main.rs` does: `Preprocessor::process` ->
   `Lexer::tokenize` -> `Parser::parse`, then `SymbolTableBuilder`,
   `ControlFlowGraphBuilder`, `TransformEngine::apply_pass`,
   `CodeGenerator::generate`, `CobolProgram::semantically_equivalent`, and
   `FormatNormalizer`. Text outputs are the same strings the CLI prints
   (`{:#?}` for the AST, symbol table and CFG).
2. **wasm-bindgen.** Every export returns one JSON string:
   `{ ok, operation, output, kind, error: {stage, message, line, column},
   diagnostics, stats: {tokens, program, format, round_trip}, tokens }`.
   Exports:

   | Export | CLI equivalent |
   |---|---|
   | `parse(source)` | `cobol-transform parse FILE` |
   | `tokens(source)` | `cobol-transform FILE --diagnostics` |
   | `dump_ast(source)` | `--dump-ast` |
   | `dump_symbols(source)` | `--dump-symbols` |
   | `dump_cfg(source)` | `--dump-cfg` |
   | `generate(source, passes)` | `cobol-transform transform FILE -p ...` (passes comma-separated) |
   | `round_trip(source)` | `cobol-transform round-trip FILE` |
   | `detect_format(source)` | `FormatNormalizer::detect_format` (no CLI flag) |
   | `normalize_format(source, format)` | `FormatNormalizer::normalize` (no CLI flag) |
   | `run(operation, source, options_json)` | dispatcher used by the UI; options: `file_name`, `passes`, `format`, `normalize_first` |
   | `version()` | wrapper version, operation names, known passes |

   Lexer errors carry a position in their message (`... at LINE:COL`); parser
   errors do not, so the wrapper reads the parser's current token location
   after a failure. Both are mapped back to the original line through the
   preprocessor's source map.
3. **Tailwind UI.** `web/src/index.html` + `web/src/app.js` (no framework, no
   bundler). `web/src/styles.css` is compiled by the Tailwind v4 CLI into a
   static `dist/app.css`; no CDN is used at runtime. The only npm dependencies
   are `tailwindcss` and `@tailwindcss/cli`.

### Changes to the transformer crate

One line: `Parser::current_location` in `cobol-transformer/src/parser.rs` was
made `pub` so the wrapper can report where a parse failed. No code needed to be
`cfg`-gated for `wasm32-unknown-unknown`: the library compiles as is. The
preprocessor's `std::fs` calls (for `COPY`) compile but have no file system in
the browser, so `COPY` members are reported as not found.

## What the demo shows

- Sample picker with real files copied byte-for-byte from the repo at build
  time: `cobol-transformer/tests/fixtures/hello.cob`, `LEDGER_POST.cbl` and
  `cobol-law-kernel.cob` from `sources/` (these three parse, generate and pass
  round-trip validation), and `cobalt/REQPARSE.cbl`, `vault_treasury.cbl`,
  `mamari.cbl` and `gpu-cobol/examples/vector-add.cbl`, which stop with a lexer
  error today. They are kept because they show the error display and where
  the transformer's coverage ends.
- Operation selector and options: transform passes, source format, optional
  format normalisation before lexing, run on edit.
- Output as a collapsible tree (built from the Rust `{:#?}` text), a token
  table (click a row to jump to the token), raw text, or the raw JSON.
- Error panel with stage, message, line and column, a "Go to line" action, and
  the failing line highlighted in the gutter.
- Dark/light toggle (remembers the choice), phone-width layout, deep links
  such as `?sample=hello&op=round-trip`.

## Verification

`npm run verify` (also run in CI) loads `web/dist/pkg` in Node, calls each
export on `hello.cob` and checks the results (program `HELLO`, 49 tokens,
generated code contains `PROGRAM-ID. HELLO.` and `DISPLAY GREETING.`, symbols
include `GREETING` and `MAIN-PARA`, round trip `PASSED`, lexer and parser
errors carry line/column). It then serves `web/dist` under
`/cobalt-transformer-hackathon/`, fetches the page and every asset, checks that
all references are relative and that each sample is byte-identical to its
source file.

## Agent-run log

There is no `docs/agent-runs/` directory in this repository, so no agent-run
log is published here.

## Limitations

These are properties of the current transformer, shown as-is by the playground:

- The lexer rejects several characters common in the collected programs
  (`=`, `<`, `>`, non-ASCII text in comments), so many files under `sources/`
  stop at the lexer. Fixed-format `*` comments are only recognised in
  column 1.
- The code generator emits the IDENTIFICATION DIVISION, an ENVIRONMENT
  DIVISION header, WORKING-STORAGE items and PROCEDURE paragraphs, with a
  subset of statements (MOVE, DISPLAY, ACCEPT, IF, PERFORM,
  STOP RUN, GOBACK); other statements are dropped from generated output.
- The `normalize` and `modernize` passes are stubs in `transform.rs`, so they
  do not change the output. Unknown pass names are ignored by the crate.
- The CFG builder returns a graph with a single entry block.
- `COPY` cannot be resolved in the browser.
- Three unit tests in the crate fail on the current sources, independent of
  this work (`lexer::tests::test_lex_level_number`,
  `lexer::tests::test_lex_keywords`,
  `preprocessor::tests::test_replace_statement`); the integration tests pass.
- GitHub Pages: the repository is private on GitHub Free, so the Pages deploy
  in `.github/workflows/pages.yml` will not publish until the repository is made
  public or the account is upgraded. The built site is committed in `web/dist`.

## Frontend debugging

From web/, run npm run verify:browser (install Chromium once with npx playwright install chromium). This exercises Execute, keyboard execution, failed runtime downloads, missing samples, queued input and stale sample requests in a real browser. npm run verify checks the Wasm wrapper and static assets; its sample checks establish source fidelity, not that all sample dialects parse.

Use npm run serve and open http://127.0.0.1:8080/?debug=1. A failed runtime download now reports Cannot execute rather than queuing forever. Failed or stale sample downloads preserve the editor. Compiler diagnostics remain visible separately from loading errors.
